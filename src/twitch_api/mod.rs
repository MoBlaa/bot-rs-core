use std::fmt::{Display, Formatter};
use core::fmt;
use url::Url;
use std::str::FromStr;
use crate::utils::rand_alphanumeric;

/// Configuring which authentication method should be used.
/// "token" = OAuth Implicit Code Flow,
/// "code" = OAuth Authorization Code Flow,
/// "client_credentials" = OAuth Client Credentials Flow
pub const ENV_TWITCH_AUTH: &str = "BRS_TWITCH_AUTH";
pub const ENV_TWITCH_CLIENT_ID: &str = "BRS_TWITCH_CLIENT_ID";
pub const ENV_TWITCH_CLIENT_SECRET: &str = "BRS_TWITCH_CLIENT_SECRET";
/// Optional Comma separated List of scopes defined at [](https://dev.twitch.tv/docs/authentication/#scopes). Defaults to: `["channel:moderate","chat:edit","chat:read","user:edit:follows","user_follows_edit"]`
pub const ENV_TWITCH_SCOPES: &str = "BRS_TWITCH_SCOPES";

static REDIRECT_URI: &str = "http://localhost:4334/";
static DEFAULT_SCOPES: [&str;5] = ["channel:moderate","chat:edit","chat:read","user:edit:follows","user_follows_edit"];

pub enum UserInfo {
    Twitch {
        login: String,
        user_id: String
    }
}

impl UserInfo {
    pub fn name(&self) -> String {
        match self {
            UserInfo::Twitch {login, ..} => login.clone()
        }
    }
}

pub struct Authentication {
    pub auth: AuthRequest,
    pub token: Option<String>,
    pub user_info: Option<UserInfo>
}

impl From<AuthRequest> for Authentication {
    fn from(auth: AuthRequest) -> Self {
        Authentication {
            auth,
            token: None,
            user_info: None
        }
    }
}

pub enum AuthRequest {
    /// [](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#oauth-implicit-code-flow) requiring GET.
    ImplicitCode {
        client_id: String,
        redirect_uri: Url,
        scope: Vec<String>,
        state: String,
        force_verify: bool
    },
    /// [](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#oauth-authorization-code-flow) requiring GET.
    AuthorizationCode {
        client_id: String,
        redirect_uri: Url,
        scope: Vec<String>,
        state: String,
        force_verify: bool,
    },
    /// [](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#oauth-client-credentials-flow) requiring POST.
    ClientCredentials {
        client_id: String,
        client_secret: String,
        scope: Vec<String>,
    }
}

impl Default for AuthRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthRequest {
    pub fn new() -> AuthRequest {
        let auth_type = std::env::var(ENV_TWITCH_AUTH)
            .unwrap_or_else(|arg| {
                warn!("Error fetching envvar: {}", arg);
                "token".to_string()
            });
        let client_id = std::env::var(ENV_TWITCH_CLIENT_ID).expect("missing client id");
        let scope = std::env::var(ENV_TWITCH_SCOPES)
            .map(|val| {
                let scopes: Vec<&str> = val.split(',').collect::<Vec<_>>();
                scopes.iter().map(|scope| scope.to_string()).collect()
            })
            .unwrap_or_else(|arg| {
                warn!("Error fetchng envvar: {}", arg);
                DEFAULT_SCOPES.iter().map(|scope| scope.to_string()).collect()
            });

        match auth_type.as_str() {
            "token" => AuthRequest::ImplicitCode {
                client_id,
                redirect_uri: Url::from_str(REDIRECT_URI).expect("invalid redirect_uri"),
                scope,
                state: rand_alphanumeric(30),
                force_verify: true
            },
            "code" => AuthRequest::AuthorizationCode {
                client_id,
                redirect_uri: Url::from_str(REDIRECT_URI).expect("invalid redirect_uri"),
                scope,
                state: rand_alphanumeric(30),
                force_verify: true
            },
            "client_credentials" => {
                let client_secret = std::env::var(ENV_TWITCH_CLIENT_SECRET)
                    .expect("client-secret required for client_credential authentication with twitch");

                AuthRequest::ClientCredentials {
                    client_id,
                    client_secret,
                    scope
                }
            },
            t => panic!("Unsupported twitch authentication Type: {}", t)
        }
    }

    pub fn client_id(&self) -> &String {
        match self {
            AuthRequest::ImplicitCode {client_id, ..} => client_id,
            AuthRequest::AuthorizationCode { client_id, ..} => client_id,
            AuthRequest::ClientCredentials {client_id, ..} => client_id
        }
    }
}

impl Display for AuthRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AuthRequest::ImplicitCode {
                client_id,
                redirect_uri,
                scope,
                state,
                force_verify
            } => {
                let url = format!("https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri={}&response_type=token&scope={}&force_verify={}&state={}",
                                  client_id, redirect_uri, scope.join("%20"), force_verify, state);
                let url = Url::from_str(&url)
                    .expect("invalid auth url");
                write!(f, "{}", url)
            },
            AuthRequest::AuthorizationCode {
                client_id,
                redirect_uri,
                scope,
                state,
                force_verify
            } => {
                let url = format!("https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&force_verify={}&state={}", client_id, redirect_uri, scope.join("%20"), force_verify, state);
                let url = Url::from_str(&url)
                    .expect("invalid auth url");

                write!(f, "{}", url)
            },
            AuthRequest::ClientCredentials {
                client_id,
                client_secret,
                scope
            } => {
                let url = format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials&scope={}", client_id, client_secret, scope.join("%20"));
                let url = Url::from_str(&url)
                    .expect("invalid auth url");

                write!(f, "{}", url)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::twitch_api::AuthRequest;
    use url::Url;
    use std::str::FromStr;

    #[test]
    fn test_format() {
        let auth = AuthRequest::ImplicitCode {
            client_id: "someclientid".to_string(),
            redirect_uri: Url::from_str("https://localhost:4334/").unwrap(),
            scope: vec!["scope1".to_string(), "scope2".to_string()],
            force_verify: true,
            state: "abcdefghijklmnopqrstuvwxyz123456789".to_string()
        };
        assert_eq!(auth.to_string(), "https://id.twitch.tv/oauth2/authorize?client_id=someclientid&redirect_uri=https://localhost:4334/&response_type=token&scope=scope1%20scope2&force_verify=true&state=abcdefghijklmnopqrstuvwxyz123456789")
    }
}
