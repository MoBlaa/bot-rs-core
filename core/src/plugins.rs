use crate::plugin::{
    CommandDeclaration, InvocationError, PluginInfo, PluginProxy, PluginRegistrar, StreamablePlugin,
};
use crate::{Message, CORE_VERSION, RUSTC_VERSION};
use async_trait::async_trait;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::future::join_all;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use libloading::Library;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs, io};

// Contains all loaded Plugins.
#[derive(Default, Debug)]
pub struct Plugins {
    commands: Vec<PluginProxy>,
    libraries: Vec<Arc<Option<Library>>>,
}

impl Plugins {
    pub fn new() -> Plugins {
        Plugins {
            commands: Vec::new(),
            libraries: Vec::new(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<impl StreamablePlugin> {
        self.commands.iter()
    }

    pub fn load_dir(&mut self, libraries_root: PathBuf) -> io::Result<()> {
        if !libraries_root.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "libraries root is not a directory",
            ));
        }

        for entry in fs::read_dir(libraries_root)? {
            self.load_file(entry?.path())?;
        }

        if self.libraries.is_empty() {
            warn!("No plugins loaded!");
        }

        Ok(())
    }

    pub fn load_file(&mut self, entry: PathBuf) -> io::Result<()> {
        if entry.exists() {
            if entry.is_file() {
                if let Some(extension) = entry.extension() {
                    if extension == "so" || extension == "dll" {
                        debug!("Trying to load plugin-file {}", entry.to_str().unwrap());
                        unsafe { self.load(entry)? };
                    }
                }
            }
        } else {
            panic!("File doesn't exist: '{}'", entry.display());
        }
        Ok(())
    }

    /// # Safety
    ///
    /// This function should only be called with a valid path to a library file.
    unsafe fn load<P: AsRef<OsStr>>(&mut self, library_path: P) -> io::Result<()> {
        // load the library into memory
        let library = Library::new(library_path).expect("failed to create new library");

        // get a pointer to the plugin_declaration symbol.
        let decl = match library.get::<*mut CommandDeclaration>(b"command_declaration\0") {
            Ok(decl) => decl.read(),
            Err(err) => {
                warn!("failed to load command_declaration skipping; {}", err);
                return Ok(());
            }
        };

        // version checks to prevent accidental ABI incompatibilities
        if decl.rustc_version != RUSTC_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "RUSTC version mismatch; botrs: {}, plugin: {}",
                    RUSTC_VERSION, decl.rustc_version
                ),
            ));
        }
        if decl.core_version != CORE_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "CORE version mismatch; botrs: {}, plugin: {}",
                    CORE_VERSION, decl.core_version
                ),
            ));
        }
        trace!("RUSTC and CORE versions match!");

        let library = Arc::new(Some(library));

        let mut registrar = Box::new(PluginRegistrar::new(Arc::clone(&library)));

        (decl.register)(&mut registrar);

        // add all loaded plugins to the functions map
        self.commands.extend(registrar.commands);
        // and make sure Commands keeps a reference to the library
        self.libraries.push(library);

        Ok(())
    }
}

#[async_trait]
impl StreamablePlugin for Plugins {
    async fn stream(
        &self,
        mut input: UnboundedReceiver<Message>,
        output: UnboundedSender<Vec<Message>>,
    ) -> Result<(), InvocationError> {
        let mut channel_inputs = Vec::with_capacity(self.commands.len());
        let mut streams = Vec::with_capacity(self.commands.len());
        for cmd in self.commands.iter() {
            let (write, read) = unbounded();
            let stream = cmd.stream(read, output.clone());
            channel_inputs.push(write);
            streams.push(stream);
        }
        tokio::spawn(async move {
            while let Some(msg) = input.next().await {
                let mut sends = Vec::with_capacity(channel_inputs.len());
                for sender in channel_inputs.iter_mut() {
                    sends.push(sender.send(msg.clone()));
                }
                // Actually send to all channels/commands
                join_all(sends).await;
            }
        });
        join_all(streams).await;
        Ok(())
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Bot-RS Core".to_string(),
            version: CORE_VERSION.to_string(),
            authors: env!("CARGO_PKG_AUTHORS").to_string(),
            repo: option_env!("CARGO_PKG_REPOSITORY").map(|repo| repo.to_string()),
            commands: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::plugin::{InvocationError, Plugin, PluginInfo, PluginProxy, StreamablePlugin};
    use crate::plugins::Plugins;
    use crate::Message;
    use async_trait::async_trait;
    use bot_rs_core_derive::*;
    use futures::{SinkExt, StreamExt};
    use std::sync::Arc;
    use test::Bencher;
    use tokio::runtime::{Builder, Runtime};

    #[derive(Debug, StreamablePlugin)]
    struct TestCommand;

    #[async_trait]
    impl Plugin for TestCommand {
        async fn call(&self, _message: Message) -> Result<Vec<Message>, InvocationError> {
            Ok(Vec::new())
        }

        fn info(&self) -> PluginInfo {
            PluginInfo {
                name: "".to_string(),
                version: "".to_string(),
                authors: "".to_string(),
                repo: None,
                commands: vec![],
            }
        }
    }

    fn bench_plugins(b: &mut Bencher, mut runtime: Runtime, plugin_count: usize, load: usize) {
        let mut raw_plugins = Vec::with_capacity(plugin_count);
        for _ in 0..plugin_count {
                raw_plugins.push(PluginProxy::from(Arc::new(TestCommand)));
        }
        let plugins = Plugins {
            commands: raw_plugins,
            libraries: vec![],
        };
        let (mut input_sender, input_receiver) = futures::channel::mpsc::unbounded::<Message>();
        let (output_sender, mut output_receiver) =
            futures::channel::mpsc::unbounded::<Vec<Message>>();

        runtime.spawn(async move {
            plugins.stream(input_receiver, output_sender).await.unwrap();
        });

        let message = Message::Irc(irc_rust::Message::from("PRIVMSG :hello"));

        b.iter(|| {
            runtime.block_on(async {
                for _ in 0..load {
                    input_sender.send(message.clone()).await.unwrap();
                }
                for _ in 0..(plugin_count * load) {
                    let result = output_receiver.next().await;
                    assert!(result.is_some());
                    assert!(result.unwrap().is_empty());
                }
            });
        });
    }

    #[bench]
    fn bench_1_plugin_basic_scheduler(b: &mut Bencher) {
        let runtime = Builder::new()
            .basic_scheduler()
            .build()
            .expect("failed to build test runtime");
        bench_plugins(b, runtime, 1, 1);
    }

    #[bench]
    fn bench_1_plugin_threaded_scheduler(b: &mut Bencher) {
        let runtime = Builder::new()
            .threaded_scheduler()
            .build()
            .expect("failed to build test runtime");
        bench_plugins(b, runtime, 1, 1);
    }

    #[bench]
    fn bench_1_plugin_basic_scheduler_100_load(b: &mut Bencher) {
        let runtime = Builder::new()
            .basic_scheduler()
            .build()
            .expect("failed to build test runtime");
        bench_plugins(b, runtime, 1, 100);
    }

    #[bench]
    fn bench_1_plugin_threaded_scheduler_100_load(b: &mut Bencher) {
        let runtime = Builder::new()
            .threaded_scheduler()
            .build()
            .expect("failed to build test runtime");
        bench_plugins(b, runtime, 1, 100);
    }

    #[bench]
    fn bench_64_plugin_basic_scheduler(b: &mut Bencher) {
        let runtime = Builder::new()
            .basic_scheduler()
            .build()
            .expect("failed to build test runtime");
        bench_plugins(b, runtime, 64, 1);
    }

    #[bench]
    fn bench_64_plugin_threaded_scheduler(b: &mut Bencher) {
        let runtime = Builder::new()
            .threaded_scheduler()
            .build()
            .expect("failed to build test runtime");
        bench_plugins(b, runtime, 64, 1);
    }

    #[bench]
    fn bench_64_plugin_basic_scheduler_100_load(b: &mut Bencher) {
        let runtime = Builder::new()
            .basic_scheduler()
            .build()
            .expect("failed to build test runtime");
        bench_plugins(b, runtime, 64, 100);
    }

    #[bench]
    fn bench_64_plugin_threaded_scheduler_10msload(b: &mut Bencher) {
        let runtime = Builder::new()
            .threaded_scheduler()
            .build()
            .expect("failed to build test runtime");
        bench_plugins(b, runtime, 64, 100);
    }
}
