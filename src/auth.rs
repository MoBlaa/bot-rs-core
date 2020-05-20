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

    pub fn id(&self) -> String {
        match self {
            UserInfo::Twitch {user_id, ..} => user_id.clone(),
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

impl Display for Credentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
    BadClientId
}

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
        assert_eq!(Credentials::from("oauth:thisisatoken"), Credentials::OAuthToken {token: "thisisatoken".to_string()});
    }
}
