use std::sync::Arc;
use crate::plugin::{StreamablePlugin, InvocationError, PluginInfo, CommandDeclaration, PluginProxy, PluginRegistrar};
use libloading::Library;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use crate::{Message, RUSTC_VERSION, CORE_VERSION};
use std::path::PathBuf;
use std::{io, fs};
use std::ffi::OsStr;
use futures::stream::StreamExt;
use futures::sink::SinkExt;
use futures::future::join_all;

// Contains all loaded Plugins.
#[derive(Default)]
pub struct Plugins {
    commands: Vec<PluginProxy>,
    libraries: Vec<Arc<Library>>,
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
        let library = Arc::new(Library::new(library_path)
            .expect("failed to create new library")
        );

        // get a pointer to the plugin_declaration symbol.
        let decl = match library
            .get::<*mut CommandDeclaration>(b"command_declaration\0") {
            Ok(decl) => decl.read(),
            Err(err) => {
                warn!("failed to load command_declaration skipping; {}", err);
                return Ok(());
            }
        };

        // version checks to prevent accidental ABI incompatibilities
        if decl.rustc_version != RUSTC_VERSION
        {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("RUSTC version mismatch; botrs: {}, plugin: {}", RUSTC_VERSION, decl.rustc_version),
            ));
        }
        if decl.core_version != CORE_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("CORE version mismatch; botrs: {}, plugin: {}", CORE_VERSION, decl.core_version),
            ));
        }
        trace!("RUSTC and CORE versions match!");

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
    async fn stream(&self,
                    mut input: UnboundedReceiver<Message>,
                    output: UnboundedSender<Vec<Message>>) -> Result<(), InvocationError> {
        let mut channel_inputs = Vec::with_capacity(self.commands.len());
        let mut streams = Vec::with_capacity(self.commands.len());
        for cmd in self.commands.iter() {
            let (write, read) = unbounded();
            let stream = cmd.stream(read, output.clone());
            channel_inputs.push(write);
            streams.push(stream);
        }
        while let Some(msg) = input.next().await {
            let mut sends = Vec::with_capacity(channel_inputs.len());
            for sender in channel_inputs.iter_mut() {
                sends.push(sender.send(msg.clone()));
            }
            // Actually send to all channels/commands
            join_all(sends).await;
        }
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
