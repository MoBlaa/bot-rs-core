use std::convert::TryFrom;
use std::fmt;

#[derive(PartialEq, Eq, Debug, Hash, Clone, Serialize, Deserialize)]
pub enum Platform {
    Twitch,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum InvalidIrcMessageError<'a> {
    MissingTags(&'a irc_rust::Message),
    MissingUserId(&'a irc_rust::Message),
    MissingPrefix(&'a irc_rust::Message),
}

impl fmt::Display for InvalidIrcMessageError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidIrcMessageError::MissingTags(msg) => {
                write!(f, "Missing Tags in IRC message: {}", msg)
            }
            InvalidIrcMessageError::MissingUserId(msg) => {
                write!(f, "Missing Tag 'user-id' in IRC tags of message: {}", msg)
            }
            InvalidIrcMessageError::MissingPrefix(msg) => {
                write!(f, "Missing prefix in IRC message: {}", msg)
            }
        }
    }
}

/// Generic enum for storing userinfo. Contains platform local information and
/// creates an API to access these data in a unified and platform independent way.
///
/// Can be created directly but should be derived with its [From] and [TryFrom] implementations.
/// Implements [From] for some api data structs if `features = ["twitch-api"]` is enabled.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, Hash)]
pub enum UserInfo {
    Twitch { name: String, id: String },
    None,
}

impl UserInfo {
    /// Returns the Name of the user on the corresponding platform.
    pub fn get_platform_name(&self) -> Option<&String> {
        match self {
            UserInfo::Twitch { name: login, .. } => Some(login),
            UserInfo::None => None,
        }
    }

    /// Returns the id of the user on the corresponding platform.
    pub fn get_platform_id(&self) -> Option<&String> {
        match self {
            UserInfo::Twitch { id: user_id, .. } => Some(user_id),
            UserInfo::None => None,
        }
    }

    /// Transforms the [UserInfo] to a unique id over all platforms.
    pub fn to_global_id(&self) -> String {
        match self {
            UserInfo::Twitch { id: user_id, .. } => format!("twitch#{}", user_id),
            UserInfo::None => String::new(),
        }
    }
}

#[cfg(feature = "twitch-api")]
impl From<crate::twitch_api::users::UserRes> for UserInfo {
    fn from(res: crate::twitch_api::users::UserRes) -> Self {
        UserInfo::Twitch {
            name: res.name,
            id: res.id,
        }
    }
}

#[cfg(feature = "twitch-api")]
impl From<&crate::twitch_api::users::UserRes> for UserInfo {
    fn from(res: &crate::twitch_api::users::UserRes) -> Self {
        UserInfo::Twitch {
            name: res.name.clone(),
            id: res.id.clone(),
        }
    }
}

impl<'a> TryFrom<&'a irc_rust::Message> for UserInfo {
    type Error = InvalidIrcMessageError<'a>;

    fn try_from(irc_message: &'a irc_rust::Message) -> Result<Self, Self::Error> {
        let tags = irc_message
            .tags()
            .expect("invalid irc message")
            .ok_or(InvalidIrcMessageError::MissingTags(irc_message))?;

        let user_id = tags
            .get("user-id")
            .map(|id| id.to_string())
            .ok_or(InvalidIrcMessageError::MissingUserId(irc_message))?;

        let username = irc_message
            .prefix()
            .expect("invalid irc message")
            .map(|prefix| prefix.name().to_string())
            // Sometimes (USERNOTICE) the username is contained as login tag
            .or_else(|| tags.get("login").map(|val| val.to_string()))
            .ok_or(InvalidIrcMessageError::MissingPrefix(irc_message))?;

        Ok(UserInfo::Twitch {
            name: username,
            id: user_id,
        })
    }
}

/// Platform independent Credentials enum.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Hash, Ord, PartialOrd)]
pub enum Credentials {
    OAuthToken { token: String },
    None,
}

impl fmt::Display for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Credentials::OAuthToken { token } => write!(f, "oauth:{}", token),
            Credentials::None => write!(f, "NONE"),
        }
    }
}

impl<S> From<S> for Credentials
    where
        S: AsRef<str>,
{
    fn from(t: S) -> Self {
        let s_t = t.as_ref();
        if let Some(token) = s_t.strip_prefix("oauth:") {
            Credentials::OAuthToken {
                token: token.to_string(),
            }
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

#[async_trait]
pub trait Authenticator {
    async fn authenticate(&self) -> Credentials;
    async fn validate(&self, cred: &Credentials) -> Result<UserInfo, ValidationError>;
}

#[cfg(test)]
mod tests {
    use crate::auth::Credentials;

    #[test]
    fn test_credentials_from_str() {
        assert_eq!(
            Credentials::OAuthToken {
                token: "thisisatoken".to_string()
            }
                .to_string(),
            "oauth:thisisatoken"
        );
        assert_eq!(
            Credentials::from("oauth:thisisatoken"),
            Credentials::OAuthToken {
                token: "thisisatoken".to_string()
            }
        );
    }

    mod userinfo {
        use crate::auth::{InvalidIrcMessageError, UserInfo};
        use std::convert::TryFrom;

        #[test]
        fn test_platform_name() {
            let userinfo = UserInfo::Twitch {
                name: "name".to_string(),
                id: "id".to_string(),
            };
            assert_eq!(userinfo.get_platform_name(), Some(&"name".to_string()));
        }

        #[test]
        fn test_platform_id() {
            let userinfo = UserInfo::Twitch {
                name: "name".to_string(),
                id: "id".to_string(),
            };
            assert_eq!(userinfo.get_platform_id(), Some(&"id".to_string()));
        }

        #[test]
        fn test_irc_no_tags() {
            let no_tags_message = irc_rust::Message::builder("PRIVMSG").build();
            let result = UserInfo::try_from(&no_tags_message);
            assert_eq!(
                result,
                Err(InvalidIrcMessageError::MissingTags(&no_tags_message))
            );
        }

        #[test]
        fn test_irc_no_userid() {
            let no_user_id = irc_rust::Message::builder("PRIVMSG")
                .tag("id", "messageid1")
                .build();
            let result = UserInfo::try_from(&no_user_id);
            assert_eq!(
                result,
                Err(InvalidIrcMessageError::MissingUserId(&no_user_id))
            );
        }

        #[test]
        fn test_irc_no_prefix() {
            let no_user_id = irc_rust::Message::builder("PRIVMSG")
                .tag("user-id", "userid1")
                .build();
            let result = UserInfo::try_from(&no_user_id);
            assert_eq!(
                result,
                Err(InvalidIrcMessageError::MissingPrefix(&no_user_id))
            );
        }

        #[test]
        fn test_irc_with_prefix() {
            let no_user_id = irc_rust::Message::builder("PRIVMSG")
                .tag("user-id", "userid1")
                .prefix("username", None, None)
                .build();
            let result = UserInfo::try_from(&no_user_id);
            assert_eq!(
                result,
                Ok(UserInfo::Twitch {
                    name: "username".to_string(),
                    id: "userid1".to_string(),
                })
            );
        }
    }
}
