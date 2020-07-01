#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate rocket;
#[cfg(test)]
#[macro_use]
extern crate bot_rs_core_derive;

pub mod twitch_api;
pub mod auth;
pub mod config;
mod utils;

// Re-Exports for a clean API
pub use config::plugins::*;
pub use config::profile::*;

use core::fmt;
use std::fmt::{Display, Formatter};

use irc_rust::message::Message as IrcMessage;

pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const RUSTC_VERSION: &str = env!("RUSTC_VERSION");

pub const ENV_JOINED_CHANNELS: &str = "BRS_JOINED_CHANNELS";

#[derive(Serialize, Deserialize, Clone)]
pub enum Message {
    Irc(IrcMessage)
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Message::Irc(msg) => {
                let str_msg = msg.to_string();
                let byte_len = str_msg.bytes().len();
                if byte_len > 512 {
                    error!("Raw IRC Message exceeds exceeds 512 Byte length: {}", byte_len);
                    Err(fmt::Error)
                } else {
                    write!(f, "{}", msg)
                }
            }
        }
    }
}
