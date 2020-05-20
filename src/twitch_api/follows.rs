use chrono::{NaiveDateTime, NaiveDate};

#[derive(Serialize, Deserialize)]
pub struct Follow {
    pub from_id: String,
    pub from_name: String,
    pub to_id: String,
    pub to_name: String,
    followed_at: String
}

impl Follow {
    pub fn followed_at(&self) -> NaiveDate {
        NaiveDateTime::parse_from_str(&self.followed_at, "%Y-%m-%d'T'%H:%M:%s'Z'")
            .map(|dt|dt.date())
            .expect("invalid date format")
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetFollowsRes {
    pub total: usize,
    pub data: Vec<Follow>
}
