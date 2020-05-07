#[macro_use]
extern crate log;

use std::rc::Rc;
use std::collections::HashMap;
use std::{io, fs};
use std::ffi::OsStr;
use std::path::PathBuf;
use libloading::Library;

pub static CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");

pub trait Command {
    /// Calls the command. Like Argv the args contain the name
    /// of the command as first element.
    fn call(&self, args: &[String]) -> Result<String, InvocationError>;

    fn info(&self) -> String;
}

#[derive(Debug)]
pub enum InvocationError {
    InvalidArgumentCount { expected: usize, found: usize },
    Other { msg: String }
}

impl<S: ToString> From<S> for InvocationError {
    fn from(other: S) -> Self {
        InvocationError::Other {
            msg: other.to_string()
        }
    }
}

pub struct CommandDeclaration {
    pub rustc_version: &'static str,
    pub core_version: &'static str,
    pub register: unsafe extern "C" fn(&mut dyn CommandRegistrar),
}

pub trait CommandRegistrar {
    fn register_command(&mut self, name: &[&str], function: Rc<dyn Command>);
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
    command: Rc<dyn Command>,
    _lib: Rc<Library>,
}

impl Command for CommandProxy {
    fn call(&self, args: &[String]) -> Result<String, InvocationError> {
        self.command.call(args)
    }

    fn info(&self) -> String {
        self.command.info()
    }
}

// Contains all loaded Commands
#[derive(Default)]
pub struct Commands {
    commands: HashMap<String, CommandProxy>,
    libraries: Vec<Rc<Library>>,
}

impl Commands {
    pub fn new() -> Commands {
        Commands {
            commands: HashMap::new(),
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
        Ok(())
    }

    /// # Safety
    ///
    /// This function should only be called with a valid path to a library file.
    unsafe fn load<P: AsRef<OsStr>>(&mut self, library_path: P) -> io::Result<()> {
        // load the library into memory
        let library = Rc::new(Library::new(library_path)
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

        let mut registrar = SimpleRegistrar::new(Rc::clone(&library));

        (decl.register)(&mut registrar);

        // add all loaded plugins to the functions map
        self.commands.extend(registrar.commands);
        // and make sure Commands keeps a reference to the library
        self.libraries.push(library);

        Ok(())
    }
}

impl Command for Commands {
    fn call(&self, arguments: &[String]) -> Result<String, InvocationError> {
        self.commands
            .get(&arguments[0])
            .ok_or_else(|| format!("\"{}\" not found", &arguments[0]))?
            .call(arguments)
    }

    fn info(&self) -> String {
        format!("Bot-RS Core {}", CORE_VERSION)
    }
}

struct SimpleRegistrar {
    commands: HashMap<String, CommandProxy>,
    lib: Rc<Library>,
}

impl SimpleRegistrar {
    fn new(lib: Rc<Library>) -> SimpleRegistrar {
        SimpleRegistrar {
            lib,
            commands: HashMap::default(),
        }
    }
}

impl CommandRegistrar for SimpleRegistrar {
    fn register_command(&mut self, names: &[&str], command: Rc<dyn Command>) {
        for name in names {
            let proxy = CommandProxy {
                command: Rc::clone(&command),
                _lib: Rc::clone(&self.lib),
            };
            if let Some(old) = self.commands.insert(name.to_string(), proxy) {
                warn!("multiple commands with name '{}'; using '{}' (overwritten '{}')", name, command.info(), old.info());
            }
        }
    }
}
