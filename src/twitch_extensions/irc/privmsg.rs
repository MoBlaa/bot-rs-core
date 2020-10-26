use crate::auth::UserInfo;
use irc_rust::{Message, Params};
use std::convert::TryFrom;
use std::ops::Deref;
use crate::{GetPropertyError, InvalidCommand};

#[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct PrivMsg(Message);

impl PrivMsg {
    fn get_tag(&self, name: &'static str) -> Result<&str, GetPropertyError> {
        let tags = self.tags()?;
        if tags.is_none() {
            return Err(GetPropertyError::MissingTags);
        }
        let tag = tags.unwrap().get(name);
        match tag {
            None => Err(GetPropertyError::MissingTag(name)),
            Some(value) => Ok(value),
        }
    }

    fn get_params(&self) -> Result<Params, GetPropertyError> {
        let params = self.params();
        if params.is_none() {
            return Err(GetPropertyError::MissingParams);
        }
        Ok(params.unwrap())
    }

    fn get_param(&self, index: usize) -> Result<Option<&str>, GetPropertyError> {
        Ok(self.get_params()?.iter().nth(index))
    }

    pub fn badge_info(&self) -> Result<Vec<&str>, GetPropertyError> {
        let info = self.get_tag("badge-info")?;
        Ok(info.split(',').collect::<Vec<_>>())
    }

    pub fn badges(&self) -> Result<Vec<&str>, GetPropertyError> {
        let raw_badges = self.get_tag("badges")?;
        Ok(raw_badges.split(',').collect::<Vec<_>>())
    }

    pub fn bits(&self) -> Result<&str, GetPropertyError> {
        self.get_tag("bits")
    }

    pub fn user_id(&self) -> Result<&str, GetPropertyError> {
        self.get_tag("user-id")
    }

    pub fn username(&self) -> Result<&str, GetPropertyError> {
        let prefix = self.prefix()?.map(|prefix| prefix.name());
        match prefix {
            None => Err(GetPropertyError::MissingPrefix),
            Some(prefix) => Ok(prefix),
        }
    }

    pub fn color(&self) -> Result<&str, GetPropertyError> {
        self.get_tag("color")
    }

    pub fn display_name(&self) -> Result<&str, GetPropertyError> {
        self.get_tag("display_name")
    }

    pub fn emotes(&self) -> Result<Vec<&str>, GetPropertyError> {
        let emotes = self.get_tag("emotes")?;
        Ok(emotes.split(',').collect::<Vec<_>>())
    }

    pub fn id(&self) -> Result<&str, GetPropertyError> {
        self.get_tag("id")
    }

    pub fn room_id(&self) -> Result<&str, GetPropertyError> {
        self.get_tag("room-id")
    }

    pub fn tmi_sent_ts(&self) -> Result<&str, GetPropertyError> {
        self.get_tag("tmi-sent-ts")
    }

    pub fn channel(&self) -> Result<&str, GetPropertyError> {
        self.get_param(0).and_then(|param| {
            if let Some(param) = param {
                Ok(param)
            } else {
                Err(GetPropertyError::MissingParam(0, "channel"))
            }
        })
    }

    pub fn message(&self) -> Result<&str, GetPropertyError> {
        let params = self.get_params()?;
        match params.trailing() {
            None => Err(GetPropertyError::MissingTrailingParam),
            Some(trailing) => Ok(trailing),
        }
    }
}

impl Deref for PrivMsg {
    type Target = Message;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<Message> for PrivMsg {
    type Error = InvalidCommand;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        let command = value.command();
        if !"PRIVMSG".eq_ignore_ascii_case(command) {
            return Err(InvalidCommand("PRIVMSG".to_string(), command.to_string()));
        }
        Ok(PrivMsg(value))
    }
}

impl TryFrom<PrivMsg> for UserInfo {
    type Error = GetPropertyError;

    fn try_from(value: PrivMsg) -> Result<Self, Self::Error> {
        Ok(UserInfo::Twitch {
            name: value.username()?.to_string(),
            id: value.user_id()?.to_string(),
        })
    }
}
