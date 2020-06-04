#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate rocket;
extern crate chrono;

pub mod twitch_api;
pub mod auth;
pub mod config;
mod utils;

// Re-Exports for a clean API
pub use config::commands::*;
pub use config::profile::*;

use core::fmt;
use std::fmt::{Display, Formatter};

use irc_rust::message::Message as IrcMessage;

pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const RUSTC_VERSION: &str = env!("RUSTC_VERSION");

pub const ENV_JOINED_CHANNELS: &str = "BRS_JOINED_CHANNELS";

pub enum Message {
    Irc(IrcMessage)
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Message::Irc(msg) => write!(f, "{}", msg)
        }
    }
}
