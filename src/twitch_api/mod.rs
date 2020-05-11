use std::fmt::{Display, Formatter};
use core::fmt;
use url::Url;
use std::str::FromStr;

pub enum Authentication {
    ImplicitCodeFlow {
        client_id: String,
        redirect_uri: Url,
        scope: Vec<&'static str>
    }
}

impl Display for Authentication {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Authentication::ImplicitCodeFlow {
                client_id,
                redirect_uri,
                scope
            } => {
                let url = format!("https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri={}&response_type=token&scope={}",
                                  client_id, redirect_uri, scope.join("%20"));
                let url = Url::from_str(&url)
                    .expect("invalid auth url");
                write!(f, "{}", url)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::twitch_api::Authentication;
    use url::Url;
    use std::str::FromStr;

    #[test]
    fn test_format() {
        let auth = Authentication::ImplicitCodeFlow {
            client_id: "someclientid".to_string(),
            redirect_uri: Url::from_str("https://localhost:4334/").unwrap(),
            scope: vec!["scope1", "scope2"]
        };
        assert_eq!(auth.to_string(), "https://id.twitch.tv/oauth2/authorize?client_id=someclientid&redirect_uri=https://localhost:4334/&response_type=token&scope=scope1%20scope2")
    }
}
