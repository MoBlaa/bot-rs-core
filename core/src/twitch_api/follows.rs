use chrono::{DateTime, NaiveDateTime};

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct Follow {
    pub from_id: String,
    pub from_name: String,
    pub to_id: String,
    pub to_name: String,
    followed_at: String,
}

impl Follow {
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

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct GetFollowsRes {
    pub total: usize,
    pub data: Vec<Follow>,
}
