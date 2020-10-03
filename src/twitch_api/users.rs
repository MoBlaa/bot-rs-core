use core::fmt;
use serde::export::Formatter;
use std::fmt::Display;

/// Builder struct for the `Get Users` endpoint of twitch ([API docs](https://dev.twitch.tv/docs/v5/reference/users#get-users)).
///
/// To create a new request you can either use [GetUserReq::from] with a Iterator over string-like
/// elements or [GetUserReq::new] if you have a vector of Strings.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct GetUsersReq {
    usernames: Vec<String>,
    base: String,
    protocol: &'static str,
}

impl GetUsersReq {
    /// Create a new request with default values `base = "api.twitch.tv", tls: true`.
    pub fn new(usernames: Vec<String>) -> GetUsersReq {
        GetUsersReq {
            usernames,
            base: "api.twitch.tv".to_string(),
            protocol: "https",
        }
    }

    /// Set the base url for the request. Defaults to `"api.twitch.tv"`.
    pub fn base(&mut self, base: String) -> &mut Self {
        self.base = base;
        self
    }

    /// Sets the protocol to use. Defaults to `true` alias `https`.
    /// If `tls` is `true` `https` will be used, `http` otherwise.
    pub fn tls(&mut self, tls: bool) -> &mut Self {
        self.protocol = if tls { "https" } else { "http" };
        self
    }
}

impl<I> From<I> for GetUsersReq
where
    I: IntoIterator,
    I::Item: ToString,
{
    fn from(iter: I) -> Self {
        GetUsersReq::new(
            iter.into_iter()
                .map(|item| item.to_string())
                .collect::<Vec<_>>(),
        )
    }
}

impl Display for GetUsersReq {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}://{}/kraken/users?login={}",
            self.protocol,
            self.base,
            self.usernames.join(",")
        )
    }
}

/// Data struct containing data returned from twitch by utilizing [GetUserReq] and represents a
/// list of users.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct GetUsersRes {
    #[serde(rename = "_total")]
    pub total: usize,
    pub users: Vec<UserRes>,
}

/// Data struct containing data returned from twitch and representing a single twitch user object.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
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
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use crate::twitch_api::users::GetUsersReq;

    #[test]
    fn test_from_iter() {
        let items = vec![
            String::from("name1"),
            String::from("name2"),
            String::from("name3"),
        ];

        let req = GetUsersReq::from(items.clone());
        assert_eq!(req.usernames, items);

        let req = GetUsersReq::from(items.iter());
        assert_eq!(req.usernames, items);
    }

    #[test]
    fn test_build_getuserreq() {
        let req = GetUsersReq::new(vec!["name1".to_string(), "name2".to_string()]);
        assert_eq!(
            req.to_string(),
            "https://api.twitch.tv/kraken/users?login=name1,name2".to_string()
        );

        let mut req = GetUsersReq::new(vec!["name1".to_string(), "name2".to_string()]);
        req.base("localhost:8080".to_string());
        assert_eq!(
            req.to_string(),
            "https://localhost:8080/kraken/users?login=name1,name2".to_string()
        );

        let mut req = GetUsersReq::new(vec!["name1".to_string(), "name2".to_string()]);
        req.base("localhost:8080".to_string()).tls(false);
        assert_eq!(
            req.to_string(),
            "http://localhost:8080/kraken/users?login=name1,name2".to_string()
        );
    }
}
