#![feature(proc_macro_hygiene, decl_macro)]

//!# Bot-RS Core
//!
//! This library is used to implement Plugins and Plugin-Loaders for the Bot-RS platform.
//!
//! ## Implementing a Plugin
//!
//! There are two kinds of plugins: **Simple Plugin** and **Streamable Plugin**.
//! While the **Simple Plugin** has a simple interface the **Streamable Plugin** enables the developer to implement a custom way to handle the incoming Stream of Messages and convert it into a Stream of outgoing messages.
//! This also means sending asynchronous messages. While **Simple Plugins** only react to messages, **Streamable Plugins** can sand messages at any time.
//!
//! The process from creating the cargo project to building the library files will be described in the following sections.
//!
//! ### Implementing a Simple Plugin
//!
//! 0. Install required tools:
//!     - [rust and cargo through rustup](https:///rustup.rs/)
//!     - [cargo-edit](https:///github.com/killercup/cargo-edit#installation): To easily edit package dependencies
//!
//! 1. Create the Cargo project with `cargo new --lib --vcs git <project name>`. And change your working directory to it (`cd hello-plugin`).
//!     ```ignore
//!     cargo new --lib --vcs git hello-plugin
//!     cd hello-plugin
//!     ```
//! 2. Add the dependency of `bot-rs-core` to the project. Every plugin has to implement the **StreamablePlugin** trait. To simplify this the "derive" feature enables the derive-macro which generates a valid StreamablePlugin implementation from a struct which implements the **Plugin** trait.
//!     The derived code requires the plugin crate to have the dependency to the [futures crate](https:///crates.io/crates/futures). Which we'll also add.
//!     As the **Plugin** trait contains async functions it's also required to have an dependency to the [async_trait crate](https:///crates.io/crates/async-trait).
//!     To handle irc messages coming from twitch (only supported platform yet) a dependency to the [irc-rust crate](https:///crates.io/crates/irc-rust) is also required.
//!     To also be able to log messages we'll use the [log crate](https:///crates.io/crates/log) with its [env_logger](https:///crates.io/crates/env_logger) implementation for simplicity.
//!     ```ignore
//!     cargo add bot-rs-core --features derive && \
//!     cargo add futures && \
//!     cargo add async-trait && \
//!     cargo add irc-rust && \
//!     cargo add log && \
//!     cargo add env_logger
//!     ```
//! 3. Add the following snippet to your `Cargo.toml` to compile the library to a loadable library file which will be loaded by a plugin-loader implementation.
//!     ```ignore
//!     [lib]
//!     crate-type = ["cdylib"]
//!     ```
//! 4. Now we can implement the actual plugin. Replace the contents of the library root file `src/lib.rs` with the following content:
//!     ```ignore
//!     // Simple logging facade
//!     #[macro_use]
//!     extern crate log;
//!
//!     // Enables the derive-macro for StreamablePlugin.
//!     #[macro_use]
//!     extern crate bot_rs_core;
//!
//!     use async_trait::async_trait;
//!
//!     use bot_rs_core::Message;
//!     use bot_rs_core::plugin::{StreamablePlugin, Plugin, InvocationError, PluginInfo, PluginRegistrar};
//!     use std::sync::Arc;
//!
//!     // Reacts to an invocation of `!hello` with `Hello, @<sender name>!`.
//!     #[derive(StreamablePlugin)]
//!     struct HelloPlugin;
//!
//!     #[async_trait]
//!     impl Plugin for HelloPlugin {
//!         async fn call(&self, msg: Message) -> Result<Vec<Message>, InvocationError> {
//!             match &msg {
//!                 Message::Irc(irc_message) => {
//!                     match irc_message.get_command() {
//!                         "PRIVMSG" => {
//!                             let params = irc_message
//!                                 .params()
//!                                 .expect("missing params in PRIVMSG");
//!                             // First param in a PRIVMSG is the channel name
//!                             let channel = params.iter()
//!                                 .next()
//!                                 .expect("no params in PRIVMSG");
//!                             let trailing = params.trailing;
//!                             if trailing.is_none() {
//!                                 // Return no messages
//!                                 return Ok(Vec::with_capacity(0));
//!                             }
//!                             let trailing = trailing.unwrap();
//!
//!                             // Check if '!hello' command was called
//!                             if !trailing.starts_with("!hello") {
//!                                 return Ok(Vec::with_capacity(0));
//!                             }
//!
//!                             let prefix = irc_message.prefix().expect("missing prefix in PRIVMSG");
//!                             let name = prefix.name();
//!
//!                             Ok(vec![Message::Irc(irc_rust::Message::builder()
//!                                 .command("PRIVMSG")
//!                                 .param(channel)
//!                                 .trailing(&format!("Hello, @{}!", name))
//!                                 .build()
//!                                 .expect("failed to build irc message")
//!                             )])
//!                         }
//!                     }
//!                 }
//!             }
//!         }
//!
//!         // Return information about the plugin for identification.
//!         fn info(&self) -> PluginInfo {
//!             PluginInfo {
//!                 name: "Hello Plugin".to_string(),
//!                 version: env!("CARGO_PKG_VERSION").to_string(),
//!                 authors: env!("CARGO_PKG_AUTHORS").to_string(),
//!                 repo: option_env!("CARGO_PKG_REPOSITORY")
//!                     .map(|repo| if repo.is_empty() { "No repo".to_string() } else { repo.to_string() }),
//!                 commands: vec!["!hello".to_string()]
//!             }
//!         }
//!     }
//!
//!     // This macro creates a static field which can be loaded by the plugin loader.
//!     export_command!(register);
//!
//!     // The plugin loading mechanism uses this function for load and register. Initializing loggers and other dependencies has to be done here.
//!     extern "C" fn register(registrar: &mut PluginRegistrar) {
//!         env_logger::init();
//!         registrar.register(Arc::new(HelloPlugin))
//!     }
//!     ```
//! 5. (Optional) Optimize plugin file for size to reduce the file size produced through `cargo build`. To do this copy the following snippet to your `Cargo.toml`:
//!     ```ignore
//!     [profile.release]
//!     lto = true
//!     codegen-units = 1
//!     opt-level = "z"
//!     ```
//!     For more infos read this guide on reducing the size of rust binaries/libraries: [](https://github.com/johnthagen/min-sized-rust).
//! 6. Building the plugin file: `cargo build --release`
//!
//! ### Implementing a StreamablePlugin
//!
//! 0. Install required tools:
//!     - [rust and cargo through rustup](https://rustup.rs/)
//!     - [cargo-edit](https://github.com/killercup/cargo-edit#installation): To easily edit package dependencies
//!
//! 1. Create the Cargo project with `cargo new --lib --vcs git <project name>`. And change your working directory to it (`cd hello-plugin`).
//!     ```ignore
//!     cargo new --lib --vcs git hello-plugin
//!     cd hello-plugin
//!     ```
//! 2. Add the dependency of `bot-rs-core` to the project.
//!     As the **StreamablePlugin** trait contains async functions it's also required to have an dependency to the [async_trait crate](https://crates.io/crates/async-trait).
//!     To handle irc messages coming from twitch (only supported platform yet) a dependency to the [irc-rust crate](https://crates.io/crates/irc-rust) is also required.
//!     To also be able to log messages we'll use the [log crate](https://crates.io/crates/log) with its [env_logger](https://crates.io/crates/env_logger) implementation for simplicity.
//!     This time we'll also require the dependency to the [futures crate](https://crates.io/crates/futures) for our StreamablePlugin implementation.
//!     ```ignore
//!     cargo add bot-rs-core && \
//!     cargo add async-trait && \
//!     cargo add irc-rust && \
//!     cargo add log && \
//!     cargo add env_logger && \
//!     cargo add futures
//!     ```
//! 3. Add the following snippet to your `Cargo.toml` to compile the library to a loadable library file which will be loaded by a plugin-loader implementation.
//!     ```ignore
//!     [lib]
//!     crate-type = ["cdylib"]
//!     ```
//! 4. Now we can implement the actual plugin. Replace the contents of the library root file `src/lib.rs` with the following content:
//!     ```ignore
//!     // Simple logging facade
//!     #[macro_use]
//!     extern crate log;
//!     extern crate futures;
//!
//!     // Enables the derive-macro for StreamablePlugin.
//!     #[macro_use]
//!     extern crate bot_rs_core;
//!
//!     use async_trait::async_trait;
//!
//!     use bot_rs_core::Message;
//!     use bot_rs_core::plugin::{StreamablePlugin, Plugin, InvocationError, PluginInfo, PluginRegistrar};
//!     use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
//!     use std::sync::Arc;
//!     use futures::{StreamExt, SinkExt};
//!
//!     /// Reacts to an invocation of `!hello` with `Hello, @<sender name>!`.
//!     struct HelloPlugin;
//!
//!     // For simplicity we'll be using the Plugin implementation showed in the previous section. But we'll implement the `StreamablePlugin` ourself this time.
//!
//!     // <insert Plugin implementation of previous section here>
//!
//!     // This implementation doesn'T require spawning new threads. This should be handled by the plugin-loader.
//!     #[async_trait]
//!     impl StreamablePlugin for HelloPlugin {
//!         async fn stream(&self,
//!             mut input: UnboundedReceiver<Message>,
//!             mut output: UnboundedSender<Vec<Message>>)
//!         -> Result<(), InvocationError> {
//!             // Read next message from input channel
//!             while let Some(msg) = input.next().await {
//!                 // Call out Plugin implementation
//!                 let results = self.call(msg).await?;
//!                 // Send the results to the output channel
//!                 output.send(results)
//!                     .await.expect("failed to send results to output");
//!            }
//!            Ok(())
//!         }
//!
//!         // Return information about the plugin for identification.
//!         fn info(&self) -> PluginInfo {
//!             Plugin::info(self)
//!         }
//!     }
//!
//!     // This macro creates a static field which can be loaded by the plugin loader.
//!     export_command!(register);
//!
//!     // The plugin loading mechanism uses this function for load and register. Initializing loggers and other dependencies has to be done here.
//!     extern "C" fn register(registrar: &mut PluginRegistrar) {
//!         env_logger::init();
//!         registrar.register(Arc::new(HelloPlugin))
//!     }
//!     ```
//! 5. (Optional) Optimize plugin file for size to reduce the file size produced through `cargo build`. To do this copy the following snippet to your `Cargo.toml`:
//!     ```ignore
//!     [profile.release]
//!     lto = true
//!     codegen-units = 1
//!     opt-level = "z"
//!     ```
//!     For more infos read this guide on reducing the size of rust binaries/libraries: [](https://github.com/johnthagen/min-sized-rust).
//! 6. Building the plugin file: `cargo build --release`
//!
//! ## Plugin-Loader
//!
//! This is currently not publicly documented. The only implementation currently present is the [botrs cli](https://github.com/MoBlaa/bot-rs-cli). To use the plugins this cli tool is required.

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
extern crate async_trait;
#[cfg(feature = "derive")]
extern crate bot_rs_core_derive;
#[cfg(feature = "plugin-loader")]
extern crate rocket;

#[cfg(feature = "default")]
pub mod auth;
#[cfg(feature = "default")]
pub mod command_access;
#[cfg(feature = "default")]
pub mod plugin;
#[cfg(feature = "plugin-loader")]
pub mod plugins;
#[cfg(feature = "default")]
pub mod profile;
#[cfg(feature = "twitch-api")]
pub mod twitch_api;
#[cfg(feature = "twitch-api")]
mod utils;

#[cfg(feature = "derive")]
pub use bot_rs_core_derive::*;

use core::fmt;
use std::fmt::{Display, Formatter};

pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const RUSTC_VERSION: &str = env!("RUSTC_VERSION");

pub const ENV_JOINED_CHANNELS: &str = "BRS_JOINED_CHANNELS";

#[derive(Serialize, Deserialize, Clone)]
pub enum Message {
    Irc(irc_rust::Message),
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Message::Irc(msg) => {
                let str_msg = msg.to_string();
                let byte_len = str_msg.bytes().len();
                if byte_len > 512 {
                    error!(
                        "Raw IRC Message exceeds exceeds 512 Byte length: {}",
                        byte_len
                    );
                    Err(fmt::Error)
                } else {
                    write!(f, "{}", msg)
                }
            }
        }
    }
}
