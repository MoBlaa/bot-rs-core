use irc_rust::message::Message;

pub static CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");

pub trait Command {
    fn call(&self, args: &[&str]) -> Result<&str, InvocationError>;

    fn help(&self) -> Option<&str> {
        None
    }
}

pub enum InvocationError {
    InvalidArgumentCount { expected: usize, found: usize },
    Other { msg: String }
}

pub struct CommandDeclaration {
    pub rustc_version: &'static str,
    pub core_version: &'static str,
    pub register: unsafe extern "C" fn(&mut dyn CommandRegistrar),
}

pub trait CommandRegistrar {
    fn register_command(&mut self, name: &str, function: Box<dyn Function>);
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
        }
    }
}
