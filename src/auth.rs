use std::fmt;
use irc_rust::message::Message as IrcMessage;
use std::convert::TryFrom;

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Hash, Clone)]
pub enum Platform {
    Twitch
}

#[derive(Debug, Clone)]
pub enum InvalidIrcMessageError<'a> {
    MissingTags(&'a IrcMessage),
    MissingUserId(&'a IrcMessage),
    MissingPrefix(&'a IrcMessage),
}

impl fmt::Display for InvalidIrcMessageError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidIrcMessageError::MissingTags(msg) => write!(f, "Missing Tags in IRC message: {}", msg),
            InvalidIrcMessageError::MissingUserId(msg) => write!(f, "Missing Tag 'user-id' in IRC tags of message: {}", msg),
            InvalidIrcMessageError::MissingPrefix(msg) => write!(f, "Missing prefix in IRC message: {}", msg)
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum UserInfo {
    Twitch {
        name: String,
        id: String,
    },
    None,
}

impl UserInfo {
    /// Returns the Name of the user on the corresponding platform.
    pub fn get_platform_name(&self) -> Option<&String> {
        match self {
            UserInfo::Twitch { name: login, .. } => Some(login),
            UserInfo::None => None
        }
    }

    /// Returns the id of the user on the corresponding platform.
    pub fn get_platform_id(&self) -> Option<&String> {
        match self {
            UserInfo::Twitch { id: user_id, .. } => Some(user_id),
            UserInfo::None => None
        }
    }

    /// Transforms the [UserInfo] to a unique id over all platforms.
    pub fn to_global_id(&self) -> String {
        match self {
            UserInfo::Twitch { id: user_id, .. } => format!("twitch#{}", user_id),
            UserInfo::None => String::new()
        }
    }
}

#[cfg(feature = "twitch-api")]
impl From<crate::twitch_api::users::UserRes> for UserInfo {
    fn from(res: crate::twitch_api::users::UserRes) -> Self {
        let username = if res.display_name.is_empty() {
            res.name
        } else {
            res.display_name
        };
        UserInfo::Twitch {
            name: username,
            id: res.id
        }
    }
}

#[cfg(feature = "twitch-api")]
impl From<&crate::twitch_api::users::UserRes> for UserInfo {
    fn from(res: &crate::twitch_api::users::UserRes) -> Self {
        let username = if res.display_name.is_empty() {
            &res.name
        } else {
            &res.display_name
        };
        UserInfo::Twitch {
            name: username.clone(),
            id: res.id.clone()
        }
    }
}

impl<'a> TryFrom<&'a IrcMessage> for UserInfo {
    type Error = InvalidIrcMessageError<'a>;

    fn try_from(irc_message: &'a IrcMessage) -> Result<Self, Self::Error> {
        let tags = irc_message.tags()
            .ok_or_else(|| InvalidIrcMessageError::MissingTags(irc_message))?;

        let user_id = tags.get("user-id")
            .map(|id| id.to_string())
            .ok_or_else(|| InvalidIrcMessageError::MissingUserId(irc_message))?;

        let username = tags.get("display-name").map(|display_name| display_name.to_string());
        let username = match username {
            None => {
                irc_message.prefix()
                    .map(|prefix| prefix.name().to_string())
                    .ok_or_else(|| InvalidIrcMessageError::MissingPrefix(irc_message))?
            }
            Some(username) => username
        };

        Ok(UserInfo::Twitch {
            name: username,
            id: user_id,
        })
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq, Debug)]
pub enum Credentials {
    OAuthToken {
        token: String
    },
    None,
}

impl fmt::Display for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Credentials::OAuthToken { token } => write!(f, "oauth:{}", token),
            Credentials::None => write!(f, "NONE")
        }
    }
}

impl<S> From<S> for Credentials where S: AsRef<str> {
    fn from(t: S) -> Self {
        let s_t = t.as_ref();
        if s_t.starts_with("oauth:") {
            Credentials::OAuthToken { token: s_t[6..].to_string() }
        } else {
            panic!("token has no supported format: {}", s_t)
        }
    }
}

#[derive(Debug)]
pub enum ValidationError {
    Invalid,
    BadClientId,
}

/// Deprecated as traits can't contain async functions currently.
#[deprecated]
pub trait Authenticator {
    fn authenticate(&self) -> Credentials;
    fn validate(&self, cred: &Credentials) -> Result<UserInfo, ValidationError>;
}

#[cfg(test)]
mod tests {
    use crate::auth::Credentials;

    #[test]
    fn test_str() {
        assert_eq!(Credentials::OAuthToken { token: "thisisatoken".to_string() }.to_string(), "oauth:thisisatoken");
        assert_eq!(Credentials::from("oauth:thisisatoken"), Credentials::OAuthToken { token: "thisisatoken".to_string() });
    }
}
