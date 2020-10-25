use crate::twitch_api::{Req, ReqV5};
use chrono::{DateTime, NaiveDateTime};

/// Request struct for the `Get Users Follows` endpoint of twitch ([API docs](https://dev.twitch.tv/docs/api/reference#get-users-follows)).
#[derive(Debug, Clone, Eq, PartialEq, Hash, Display)]
#[display(fmt = "{}", "self.url()")]
pub struct GetUsersFollowsReq {
    from_id: Option<String>,
    to_id: Option<String>,
}

impl GetUsersFollowsReq {
    pub fn new(from_id: Option<String>, to_id: Option<String>) -> Self {
        assert!(from_id.is_some() || to_id.is_some());
        GetUsersFollowsReq { from_id, to_id }
    }
}

impl Req for GetUsersFollowsReq {
    fn url(&self) -> String {
        let mut result = String::from("https://api.twitch.tv/helix/users/follows");
        let mut has_from = false;
        if let Some(from_id) = &self.from_id {
            result.push_str("?from_id=");
            result.push_str(from_id);
            has_from = true;
        }
        if let Some(to_id) = &self.to_id {
            let start = if has_from { '?' } else { '&' };
            result.push(start);
            result.push_str("to_id=");
            result.push_str(to_id);
        }
        result
    }
}

impl ReqV5 for GetUsersFollowsReq {}

/// Simple data class obtained by twitch api calls and through [serde::Deserialize].
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize, Ord, PartialOrd)]
pub struct Follow {
    pub from_id: String,
    pub from_name: String,
    pub to_id: String,
    pub to_name: String,
    followed_at: String,
}

impl Follow {
    /// Lazily parses the obtained `followed_at` field to [chrono::NaiveDateTime] values.
    pub fn followed_at(&self) -> NaiveDateTime {
        match DateTime::parse_from_rfc3339(&self.followed_at) {
            Err(why) => panic!(
                "failed to parse followed_at '{}': {}",
                self.followed_at, why
            ),
            Ok(dt) => dt.naive_utc(),
        }
    }
}

/// Data class obtained by twitch api calls and through [serde::Deserialize].
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct GetFollowsRes {
    pub total: usize,
    pub data: Vec<Follow>,
}
