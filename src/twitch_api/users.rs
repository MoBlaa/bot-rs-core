use std::fmt::Display;
use serde::export::Formatter;
use core::fmt;

pub struct GetUsersReq {
    usernames: Vec<String>
}

impl GetUsersReq {
    pub fn new(usernames: Vec<String>) -> GetUsersReq {
        GetUsersReq {
            usernames
        }
    }
}

impl Display for GetUsersReq {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "https://api.twitch.tv/kraken/users?login={}", self.usernames.join(","))
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
    pub bio: String,
    pub created_at: String,
    pub display_name: String,
    pub logo: String,
    pub name: String,
    #[serde(rename = "type")]
    pub typ: String,
    pub updated_at: String
}
