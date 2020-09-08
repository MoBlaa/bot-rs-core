use std::fmt::Display;
use serde::export::Formatter;
use core::fmt;

pub struct GetUsersReq {
    usernames: Vec<String>,
    base: String,
    protocol: &'static str
}

impl GetUsersReq {
    pub fn new(usernames: Vec<String>) -> GetUsersReq {
        GetUsersReq {
            usernames,
            base: "api.twitch.tv".to_string(),
            protocol: "https"
        }
    }

    pub fn base<S: ToString>(&mut self, base: S) -> &mut Self {
        self.base = base.to_string();
        self
    }

    pub fn tls(&mut self, tls: bool) -> &mut Self {
        self.protocol = if tls { "https" } else { "http" };
        self
    }
}

impl Display for GetUsersReq {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}://{}/kraken/users?login={}", self.protocol, self.base, self.usernames.join(","))
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetUsersRes {
    #[serde(rename = "_total")]
    pub total: usize,
    pub users: Vec<UserRes>
}

#[derive(Serialize, Deserialize)]
pub struct UserRes {
    #[serde(rename = "_id")]
    pub id: String,
    pub bio: Option<String>,
    pub created_at: String,
    pub display_name: String,
    pub logo: String,
    pub name: String,
    #[serde(rename = "type")]
    pub typ: String,
    pub updated_at: String
}
