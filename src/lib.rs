use std::rc::Rc;

pub static CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");

pub trait Command {
    /// Calls the command. Like Argv the args contain the name
    /// of the command as first element.
    fn call(&self, args: &[String]) -> Result<String, InvocationError>;
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
