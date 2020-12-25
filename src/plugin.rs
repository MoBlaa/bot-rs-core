use core::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use libloading::Library;

use crate::Message;
use std::error::Error;
use crate::context::Ctx;
use futures::lock::Mutex;

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

/// Handles single command invocations returning their result.
#[async_trait::async_trait]
pub trait Plugin: Debug + Send + Sync {
    async fn call(&mut self, ctx: Ctx, message: Message);

    async fn info(&self) -> PluginInfo;
}

/// Wrapper for any Error a Plugin generates.
#[derive(Debug, Clone)]
pub struct PluginError(Arc<dyn Error>);

/// Marker trait for all traits which are not [PluginError].
pub auto trait NotInvocErr {}

impl ! NotInvocErr for PluginError {}

impl<E: 'static + Error + NotInvocErr> From<E> for PluginError {
    fn from(val: E) -> Self {
        PluginError(Arc::new(val))
    }
}

impl Error for PluginError {}

impl Display for PluginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "InvocationError: {}", self.0)
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
    command: Arc<Mutex<dyn Plugin>>,
    _lib: Option<Arc<Library>>,
}

impl<P: Plugin + 'static> From<Arc<Mutex<P>>> for PluginProxy {
    fn from(plugin: Arc<Mutex<P>>) -> Self {
        PluginProxy {
            command: plugin,
            _lib: None,
        }
    }
}

#[async_trait::async_trait]
impl Plugin for PluginProxy {
    async fn call(&mut self, ctx: Ctx, message: Message) {
        self.command.lock().await.call(ctx, message).await;
    }

    async fn info(&self) -> PluginInfo {
        self.command.lock().await.info().await
    }
}

#[derive(Clone, Debug)]
pub struct PluginRegistrar {
    pub(crate) commands: Vec<PluginProxy>,
    lib: Option<Arc<Library>>,
}

impl PluginRegistrar {
    pub fn new(lib: Option<Arc<Library>>) -> PluginRegistrar {
        PluginRegistrar {
            lib,
            commands: Vec::new(),
        }
    }

    pub fn register(&mut self, command: Arc<Mutex<dyn Plugin>>) {
        let proxy = PluginProxy {
            command: Arc::clone(&command),
            _lib: self.lib.clone(),
        };
        self.commands.push(proxy);
    }
}

#[cfg(test)]
mod tests {
    use crate::plugin::{Plugin, PluginError, PluginInfo};
    use crate::Message;
    use async_trait::async_trait;
    use bot_rs_core_derive::*;
    use futures::{SinkExt, StreamExt};
    use test::Bencher;
    use tokio::runtime::Builder;

    use crate as bot_rs_core;
    use crate::context::Ctx;

    #[derive(Debug)]
    struct TestCommand;

    impl Plugin for TestCommand {
        fn call(&mut self, context: Ctx, _message: Message) {}

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
    async fn test_compile() -> Result<(), PluginError> {
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
