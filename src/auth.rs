use core::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum UserInfo {
    Twitch {
        login: String,
        user_id: String,
    },
    None,
}

impl UserInfo {
    pub fn name(&self) -> String {
        match self {
            UserInfo::Twitch { login, .. } => login.clone(),
            UserInfo::None => String::new()
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum Credentials {
    OAuthToken {
        token: String
    },
    None,
}

impl Credentials {
    pub fn to_header(&self) -> String {
        match self {
            Credentials::OAuthToken { token } => format!("OAuth {}", token),
            Credentials::None => panic!("tried to convert Credentials::NONE to header"),
        }
    }
}

impl Display for Credentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Credentials::OAuthToken { token } => write!(f, "oauth:{}", token),
            Credentials::None => write!(f, "NONE")
        }
    }
}

#[derive(Debug)]
pub enum ValidationError {
    Invalid,
    BadClientId
}

pub trait Authenticator {
    fn authenticate(&self) -> Credentials;
    fn validate(&self, cred: &Credentials) -> Result<UserInfo, ValidationError>;
}
