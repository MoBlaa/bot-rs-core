use core::fmt;
use std::{fs, io};
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures::future::join_all;
use futures::SinkExt;
use libloading::Library;
use tokio::stream::StreamExt;

use crate::{CORE_VERSION, Message, RUSTC_VERSION};

/// Contains information about a plugin to identify the supported commands, author information, etc.
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub authors: String,
    pub repo: Option<String>,
    pub commands: Vec<String>,
}

impl Display for PluginInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} v{}", self.name, self.version)?;
        if let Some(ref repo) = self.repo {
            write!(f, " [{}]", repo)?;
        }
        writeln!(f)?;
        writeln!(f, "Authors: {}", self.authors)?;
        if self.commands.is_empty() {
            write!(f, "Commands: None")?;
        } else {
            write!(f, "Commands: [{}]", self.commands.join(", "))?;
        }
        Ok(())
    }
}

/// Handles single command invocations immediately returning their result.
#[async_trait]
pub trait Plugin: Send + Sync {
    async fn call(&self, message: Message) -> Result<Vec<Message>, InvocationError>;

    fn info(&self) -> PluginInfo;
}

/// Allows users to create an asynchronously running stream. This allows commands
/// to send messages to the output without the need of a command invocation.
#[async_trait]
pub trait StreamablePlugin: Send + Sync {
    /// Create a new Stream sending messages into [output] and receiving messages to
    /// the returned sender.
    async fn stream(&self, input: UnboundedReceiver<Message>, output: UnboundedSender<Vec<Message>>) -> Result<(), InvocationError>;

    fn info(&self) -> PluginInfo;
}

#[derive(Debug)]
pub enum InvocationError {
    InvalidArgumentCount { expected: usize, found: usize },
    Other { msg: String },
}

impl From<String> for InvocationError {
    fn from(other: String) -> Self {
        InvocationError::Other {
            msg: other
        }
    }
}

impl From<&str> for InvocationError {
    fn from(other: &str) -> Self {
        InvocationError::Other {
            msg: other.to_string()
        }
    }
}

impl Display for InvocationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InvocationError::Other { msg } => writeln!(f, "InvocationError: {}", msg),
            InvocationError::InvalidArgumentCount { expected, found } => writeln!(f, "Invalid argument count: {} (expected {})", found, expected)
        }
    }
}

pub struct CommandDeclaration {
    pub rustc_version: &'static str,
    pub core_version: &'static str,
    pub register: unsafe extern "C" fn(&mut PluginRegistrar),
}

#[macro_export]
macro_rules! export_command {
    ($register:expr) => {
        #[doc(hidden)]
        #[no_mangle]
        pub static command_declaration: $crate::CommandDeclaration = $crate::CommandDeclaration {
            rustc_version: $crate::RUSTC_VERSION,
            core_version: $crate::CORE_VERSION,
            register: $register
        };
    }
}

pub struct PluginProxy {
    command: Arc<dyn StreamablePlugin>,
    _lib: Arc<Library>,
}

#[async_trait]
impl StreamablePlugin for PluginProxy {
    async fn stream(&self, input: UnboundedReceiver<Message>, output: UnboundedSender<Vec<Message>>) -> Result<(), InvocationError> {
        self.command.stream(input, output).await
    }

    fn info(&self) -> PluginInfo {
        self.command.info()
    }
}

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

    pub fn iter(&self) -> std::slice::Iter<PluginProxy> {
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

pub struct PluginRegistrar {
    commands: Vec<PluginProxy>,
    lib: Arc<Library>,
}

impl PluginRegistrar {
    fn new(lib: Arc<Library>) -> PluginRegistrar {
        PluginRegistrar {
            lib,
            commands: Vec::new(),
        }
    }
}

impl PluginRegistrar {
    pub fn register(&mut self, command: Arc<dyn StreamablePlugin>) {
        let proxy = PluginProxy {
            command: Arc::clone(&command),
            _lib: Arc::clone(&self.lib),
        };
        self.commands.push(proxy);
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use crate::{InvocationError, Message, Plugin, StreamablePlugin, PluginInfo};

    #[derive(StreamablePlugin)]
    struct TestCommand;

    #[async_trait]
    impl Plugin for TestCommand {
        async fn call(&self, _message: Message) -> Result<Vec<Message>, InvocationError> {
            println!("Test command called!");
            Ok(Vec::new())
        }

        fn info(&self) -> PluginInfo {
            PluginInfo {
                name: "".to_string(),
                version: "".to_string(),
                authors: "".to_string(),
                repo: None,
                commands: vec![]
            }
        }
    }

    #[tokio::test]
    async fn test_compile() -> Result<(), InvocationError> {
        let message = Message::Irc(irc_rust::message::Message::new("CMD :test".to_string()));
        let result = TestCommand.call(message).await?;
        assert!(result.is_empty());
        Ok(())
    }
}
