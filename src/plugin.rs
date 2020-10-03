use core::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use libloading::Library;

use crate::Message;
use std::error::Error;

/// Contains information about a plugin to identify the supported commands, author information, etc.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
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
pub trait StreamablePlugin: Send + Sync + Debug {
    /// Create a new Stream sending messages into **output** and receiving messages to
    /// the returned sender.
    async fn stream(
        &self,
        input: UnboundedReceiver<Message>,
        output: UnboundedSender<Vec<Message>>,
    ) -> Result<(), InvocationError>;

    fn info(&self) -> PluginInfo;
}

#[derive(Debug, Eq, PartialEq, Clone, Hash, Serialize, Deserialize)]
pub enum InvocationError {
    InvalidArgumentCount { expected: usize, found: usize },
    Other { msg: String },
}

impl From<String> for InvocationError {
    fn from(other: String) -> Self {
        InvocationError::Other { msg: other }
    }
}

impl From<&str> for InvocationError {
    fn from(other: &str) -> Self {
        InvocationError::Other {
            msg: other.to_string(),
        }
    }
}

impl Error for InvocationError {}

impl Display for InvocationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            InvocationError::Other { msg } => writeln!(f, "InvocationError: {}", msg),
            InvocationError::InvalidArgumentCount { expected, found } => writeln!(
                f,
                "Invalid argument count: {} (expected {})",
                found, expected
            ),
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
        pub static command_declaration: $crate::plugin::CommandDeclaration =
            $crate::plugin::CommandDeclaration {
                rustc_version: $crate::RUSTC_VERSION,
                core_version: $crate::CORE_VERSION,
                register: $register,
            };
    };
}

#[derive(Debug, Clone)]
pub(crate) struct PluginProxy {
    command: Arc<dyn StreamablePlugin>,
    _lib: Arc<Option<Library>>,
}

impl<P: StreamablePlugin + 'static> From<Arc<P>> for PluginProxy {
    fn from(plugin: Arc<P>) -> Self {
        PluginProxy {
            command: plugin,
            _lib: Arc::new(None),
        }
    }
}

#[async_trait]
impl StreamablePlugin for PluginProxy {
    async fn stream(
        &self,
        input: UnboundedReceiver<Message>,
        output: UnboundedSender<Vec<Message>>,
    ) -> Result<(), InvocationError> {
        self.command.stream(input, output).await
    }

    fn info(&self) -> PluginInfo {
        self.command.info()
    }
}

#[derive(Clone, Debug)]
pub struct PluginRegistrar {
    pub(crate) commands: Vec<PluginProxy>,
    lib: Arc<Option<Library>>,
}

impl PluginRegistrar {
    pub fn new(lib: Arc<Option<Library>>) -> PluginRegistrar {
        PluginRegistrar {
            lib,
            commands: Vec::new(),
        }
    }

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
    use crate::plugin::{InvocationError, Plugin, PluginInfo, StreamablePlugin};
    use crate::Message;
    use async_trait::async_trait;
    use bot_rs_core_derive::*;
    use futures::{SinkExt, StreamExt};
    use test::Bencher;
    use tokio::runtime::Builder;

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

    #[tokio::test]
    async fn test_compile() -> Result<(), InvocationError> {
        let message = Message::Irc(irc_rust::Message::from("CMD :test".to_string()));
        let result = TestCommand.call(message).await?;
        assert!(result.is_empty());
        Ok(())
    }

    #[bench]
    fn bench_derive_delegation(b: &mut Bencher) {
        let (mut input_sender, input_receiver) = futures::channel::mpsc::unbounded::<Message>();
        let (output_sender, mut output_receiver) =
            futures::channel::mpsc::unbounded::<Vec<Message>>();
        let mut runtime = Builder::new().basic_scheduler().build().unwrap();

        runtime.spawn(async move {
            let plugin = TestCommand;
            plugin.stream(input_receiver, output_sender).await.unwrap();
        });

        let message = Message::Irc(irc_rust::Message::builder("PRIVMSG").build());

        b.iter(|| {
            runtime
                .block_on(input_sender.send(message.clone()))
                .unwrap();
            let result = runtime.block_on(output_receiver.next());
            assert!(result.is_some());
            assert!(result.unwrap().is_empty());
        });
    }

    #[bench]
    fn bench_call(b: &mut Bencher) {
        let mut runtime = Builder::new().basic_scheduler().build().unwrap();

        let plugin = TestCommand;
        let message = Message::Irc(irc_rust::Message::builder("PRIVMSG").build());

        b.iter(|| {
            let result = runtime.block_on(plugin.call(message.clone()));
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        });
    }
}
