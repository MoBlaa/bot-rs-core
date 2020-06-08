use core::fmt;
use std::{fs, io};
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use libloading::Library;

use crate::{CORE_VERSION, Message, RUSTC_VERSION};
use tokio::task::JoinHandle;

pub trait SimpleCommand {
    /// Calls the command. Like Argv the args contain the name
    /// of the command as first element.
    fn call(&self, args: &[&str]) -> Result<Vec<String>, InvocationError>;

    fn info(&self) -> String;
}

pub trait Command {
    fn call(&self, message: Message) -> Result<Vec<Message>, InvocationError>;

    fn info(&self) -> String;
}

pub trait AsyncCommand {
    fn stream(&'static self, input: UnboundedReceiver<Message>, output: UnboundedSender<Message>) -> Option<JoinHandle<()>>;

    fn info(&self) -> String;
}

#[macro_export]
macro_rules! implement_async {
    ($type:ty) => {
        use futures::{SinkExt, StreamExt};

        impl $crate::AsyncCommand for $type {
            fn stream(&'static self, mut input: futures::channel::mpsc::UnboundedReceiver<Message>, output: futures::channel::mpsc::UnboundedSender<Message>) -> Option<tokio::task::JoinHandle<()>> {
                Some(tokio::spawn(async move {
                    while let Some(msg) = input.next().await {
                        match self.call(msg) {
                            Ok(mssgs) => for msg in mssgs {
                                output.clone().send(msg).await.unwrap();
                            },
                            Err(why) => error!("Error while calling command: {}", why)
                        }
                    }
                }))
            }

            fn info(&self) -> String {
                Command::info(self)
            }
        }
    }
}

/// Can be used to generate implementation of [IrcCommand] for traits already
/// implementing the [Command] trait.
/// TODO: Remove or fix this implementation
#[macro_export]
macro_rules! implement_irc {
    ($type:ty) => {
        use irc_rust::message::Message;
        use bot_rs_core::Profile;

        #[doc(hidden)]
        impl $type {
            fn extract_params<'a>(&self, invocation: &'a str) -> Vec<&'a str> {
                if !invocation.starts_with('!') {
                    panic!("invoked without prefixing with `!`");
                }
                let mut result = Vec::new();
                if let Some(index) = invocation.chars().position(|c| c == ' ') {
                    let name = &invocation[1..index];
                    result.push(name);
                    result.extend(invocation[index + 1..].split(' ').collect::<Vec<_>>());
                } else {
                    let name = &invocation[1..];
                    result.push(name);
                }
                result
            }
        }

        #[doc(hidden)]
        impl $crate::IrcCommand for $type {
            fn call_raw(&self, message: &Message) -> Result<Vec<Message>, $crate::InvocationError> {
                if message.command() != "PRIVMSG" {
                    return Ok(Vec::with_capacity(0));
                }
                let profile: Profile = Profile::active();
                if !profile.rights().allowed(message).unwrap_or(false) {
                    return Ok(Vec::with_capacity(0));
                }

                let tags = message.tags()
                    .expect("missing tags in message");
                let sender = &tags["display-name"];

                let params = match message.params() {
                    None => {
                        return Ok(Vec::with_capacity(0))
                    },
                    Some(params) => params
                };
                let channel = params.iter().next().expect("missing channel param");
                let trailing = params.trailing;
                if trailing.is_none() {
                    return Ok(Vec::with_capacity(0));
                }
                let trailing = trailing.unwrap().trim();
                if !trailing.starts_with('!') {
                    return Ok(Vec::with_capacity(0));
                }
                let params = $crate::IrcCommand::extract_params(self, trailing);

// TODO: filter for names before invoking
                self.call(&params)
                    .map(|result: Vec<String>| {
                        let mut messages = Vec::with_capacity(result.len());
                        for result in result {
                            let message = format!("@{}: {}", sender, result);
                            messages.push(Message::builder()
                            .command("PRIVMSG")
                            .param(channel)
                            .trailing(message.as_str())
                            .build()
                            );
                        }
                        messages
                    })
            }

            fn info(&self) -> String {
                Command::info(self)
            }
        }
    }
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
    pub register: unsafe extern "C" fn(&mut dyn CommandRegistrar),
}

pub trait CommandRegistrar {
    fn register(&mut self, command: Arc<dyn AsyncCommand + Send + Sync>);
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

struct CommandProxy {
    command: Arc<dyn AsyncCommand + Send + Sync>,
    _lib: Arc<Library>,
}

impl AsyncCommand for CommandProxy {

    fn stream(&'static self, input: UnboundedReceiver<Message>, output: UnboundedSender<Message>) -> Option<JoinHandle<()>>{
        self.command.stream(input, output)
    }

    fn info(&self) -> String {
        self.command.info()
    }
}

// Contains all loaded Commands
#[derive(Default)]
pub struct Commands {
    commands: Vec<CommandProxy>,
    libraries: Vec<Arc<Library>>,
}

impl Commands {
    pub fn new() -> Commands {
        Commands {
            commands: Vec::new(),
            libraries: Vec::new(),
        }
    }

    pub fn load_dir(&mut self, libraries_root: PathBuf) -> io::Result<()> {
        if !libraries_root.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "libraries root is not a directory",
            ));
        }

        for entry in fs::read_dir(libraries_root)? {
            let entry = entry?.path();
            if entry.is_file() {
                if let Some(extension) = entry.extension() {
                    if extension == "so" {
                        debug!("Loading plugin-file {}", entry.to_str().unwrap());
                        unsafe { self.load(entry)? };
                    }
                }
            }
        }

        if self.libraries.is_empty() {
            warn!("No plugins loaded!");
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
        let decl = library
            .get::<*mut CommandDeclaration>(b"command_declaration\0")
            .expect("failed to get command_declaration")
            .read();

        // version checks to prevent accidental ABI incompatibilities
        if decl.rustc_version != RUSTC_VERSION
            || decl.core_version != CORE_VERSION
        {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Version mismatch",
            ));
        }
        trace!("RUSTC and CORE versions match!");

        let mut registrar = SimpleRegistrar::new(Arc::clone(&library));

        (decl.register)(&mut registrar);

        // add all loaded plugins to the functions map
        self.commands.extend(registrar.commands);
        // and make sure Commands keeps a reference to the library
        self.libraries.push(library);

        Ok(())
    }
}


impl AsyncCommand for Commands {
    fn stream(&'static self, mut input: UnboundedReceiver<Message>, output: UnboundedSender<Message>) -> Option<JoinHandle<()>> {
        let mut senders = Vec::with_capacity(self.commands.len());
        let mut cmd_outputs = Vec::with_capacity(self.commands.len());
        for cmd in self.commands.iter() {
            let (sender, receiver) = unbounded();
            senders.push(sender);
            cmd_outputs.push(cmd.stream(receiver, output.clone()));
        }
        Some(tokio::spawn(async move {
            while let Some(msg) = input.next().await {
                for mut sender in senders.iter() {
                    sender.send(msg.clone()).await.unwrap();
                }
            }
        }))
    }

    fn info(&self) -> String {
        format!("Bot-RS Core {}", CORE_VERSION)
    }
}

struct SimpleRegistrar {
    commands: Vec<CommandProxy>,
    lib: Arc<Library>,
}

impl SimpleRegistrar {
    fn new(lib: Arc<Library>) -> SimpleRegistrar {
        SimpleRegistrar {
            lib,
            commands: Vec::new(),
        }
    }
}

impl CommandRegistrar for SimpleRegistrar {
    fn register(&mut self, command: Arc<dyn AsyncCommand + Send + Sync>) {
        let proxy = CommandProxy {
            command: Arc::clone(&command),
            _lib: Arc::clone(&self.lib),
        };
        self.commands.push(proxy);
    }
}
