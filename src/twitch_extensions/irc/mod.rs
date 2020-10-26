use irc_rust::{InvalidIrcFormatError, Message};
use std::fmt;
use std::error::Error;

pub mod privmsg;

pub use privmsg::*;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct InvalidCommand(pub(crate) String, pub(crate) String);

impl fmt::Display for InvalidCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Expected '{}' but got '{}'", self.0, self.1)
    }
}

impl Error for InvalidCommand {}

#[derive(Debug)]
pub enum GetPropertyError {
    MissingTags,
    InvalidFormat(InvalidIrcFormatError),
    MissingTag(&'static str),
    MissingPrefix,
    MissingParams,
    MissingParam(usize, &'static str),
    MissingTrailingParam,
}

impl fmt::Display for GetPropertyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTags => write!(f, "missing irc tags"),
            Self::InvalidFormat(err) => fmt::Display::fmt(err, f),
            Self::MissingTag(name) => write!(f, "required tag '{}' is missing", name),
            Self::MissingPrefix => write!(f, "missing irc prefix"),
            Self::MissingParams => write!(f, "missing irc params"),
            Self::MissingParam(index, name) => {
                write!(f, "missing {} parameter at index {}", name, index)
            }
            Self::MissingTrailingParam => write!(f, "missing irc trailing parameter"),
        }
    }
}

impl Error for GetPropertyError {}

impl From<InvalidIrcFormatError> for GetPropertyError {
    fn from(err: InvalidIrcFormatError) -> Self {
        Self::InvalidFormat(err)
    }
}

/// Trait extension for Irc Messages
pub trait TwitchIrcMessage {
    fn into_privmsg(self) -> Result<PrivMsg, InvalidCommand>;
}

impl TwitchIrcMessage for Message {
    fn into_privmsg(self) -> Result<PrivMsg, InvalidCommand> {
        PrivMsg::try_from(self)
    }
}
