use crate::auth::{Authenticator, Credentials, UserInfo, ValidationError};
use crate::utils::rand_alphanumeric;
use chrono::{Duration, Local};
use core::fmt;
use rocket::config::Environment;
use rocket::response::content;
use rocket::{get, post, routes, State};
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::str::FromStr;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use url::Url;

/// Configuring which authentication method should be used.
/// "token" = OAuth Implicit Code Flow,
/// "code" = OAuth Authorization Code Flow,
/// "client_credentials" = OAuth Client Credentials Flow
pub const ENV_TWITCH_AUTH: &str = "BRS_TWITCH_AUTH";
/// Name of the envvar which will contain the twitch auth token.
pub const ENV_TWITCH_TOKEN: &str = "BRS_TWITCH_TOKEN";
/// Contains a serialized version of the userInfo of the currently used bot account.
pub const ENV_TWITCH_USER_INFO: &str = "BRS_TWITCH_USERINFO";

/// Optional Comma separated List of scopes defined at [](https://dev.twitch.tv/docs/authentication/#scopes). Defaults to: `["channel:moderate","chat:edit","chat:read","user:edit:follows","user_follows_edit"]`
pub const ENV_TWITCH_SCOPES: &str = "BRS_TWITCH_SCOPES";
const TWITCH_OAUTH_HANDLER_SCRIPT: &str = include_str!("twitch_oauth.html");

static REDIRECT_URI: &str = "http://localhost:4334/";
static DEFAULT_SCOPES: [&str; 8] = [
    "channel:moderate",
    "chat:edit",
    "chat:read",
    "user:edit:follows",
    "user_follows_edit",
    "user:edit",
    "user_read",
    "whispers:edit",
];

type AuthMutex = Arc<(Mutex<Option<Credentials>>, Condvar)>;

/// Endpoint serving the html to read the OAuth fragment generated.
#[get("/")]
fn index() -> content::Html<&'static str> {
    content::Html(TWITCH_OAUTH_HANDLER_SCRIPT)
}

/// Endpoint to actually register.
#[post("/auth?<access_token>&<state>")]
fn auth_get(
    auth_req: State<AuthRequest>,
    auth: State<AuthMutex>,
    access_token: String,
    state: Option<String>,
) -> String {
    let nonce = match auth_req.inner() {
        AuthRequest::ImplicitCode { ref state, .. } => state,
        AuthRequest::AuthorizationCode { ref state, .. } => state,
        _ => "",
    };
    let state = state.expect("missing state in OAuth redirect");
    if !nonce.is_empty() && state != nonce {
        panic!("state doesn't match. Expected={}, Actual={}", nonce, state);
    }

    let (lock, cvar) = auth.as_ref();
    let mut token = lock.lock().unwrap();

    *token = Some(Credentials::OAuthToken {
        token: access_token,
    });
    cvar.notify_all();

    "Successfully obtained access token! You can close this window now..".to_string()
}

/// Struct implementing the [Authenticator] trait for Twitch.
/// This handles the OAuth authentication process with `client-id` and optional `client-secret`
/// described in [the twitch apis](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth).
///
/// The used [AuthRequest] is built from the client information on creation and from environment
/// variables. See [AuthRequest::new].
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TwitchAuthenticator {
    client_id: String,
    client_secret: Option<String>,
}

impl TwitchAuthenticator {
    pub fn new(client_id: String, client_secret: Option<String>) -> Self {
        TwitchAuthenticator {
            client_id,
            client_secret,
        }
    }
}

#[async_trait]
impl Authenticator for TwitchAuthenticator {
    async fn authenticate(&self) -> Credentials {
        let req = AuthRequest::new(self.client_id.clone(), self.client_secret.clone());
        info!(
            "For authentication please grant Nemabot access to the Bots Twitch account at: '{}'",
            req.to_string()
        );
        let auth_lock: AuthMutex = Arc::new((Mutex::new(None), Condvar::new()));

        let r_auth_lock = Arc::clone(&auth_lock);
        thread::spawn(move || {
            let cfg =
                rocket::Config::build(Environment::active().expect("missing rocket environment"))
                    .port(4334)
                    .finalize()
                    .expect("failed to build rocket config");
            rocket::custom(cfg)
                .manage(Arc::clone(&r_auth_lock))
                .manage(req)
                .mount("/", routes![index, auth_get])
                .launch();
        });

        let (lock, cvar) = &*auth_lock;
        let mut auth = lock.lock().unwrap();
        while auth.is_none() {
            auth = cvar.wait(auth).unwrap();
        }
        auth.take().unwrap()
    }

    async fn validate(&self, cred: &Credentials) -> Result<UserInfo, ValidationError> {
        let client = reqwest::Client::new();
        let header = match cred {
            Credentials::OAuthToken { token } => format!("OAuth {}", token),
            token => panic!("invalid token for twitch validation: {:?}", token),
        };

        let response = client
            .get("https://id.twitch.tv/oauth2/validate")
            .header("Authorization", &header)
            .send()
            .await
            .expect("validation request failed");
        if !response.status().is_success() {
            error!("validation request failed: {}", response.status());
            return Err(ValidationError::Invalid);
        }
        let body: String = response.text().await.expect("invalid response body");
        let body: TwitchValidation = serde_json::from_str(&body).unwrap();
        if body.client_id != self.client_id {
            error!(
                "Client-ID doesn't match the one used for auth. Expected: {}, Actual: {}",
                self.client_id, body.client_id
            );
            return Err(ValidationError::BadClientId);
        }
        let exp_dur = Duration::seconds(body.expires_in);
        let exp_date = Local::now().add(exp_dur);
        warn!("Token expires on: {}", exp_date);
        Ok(UserInfo::Twitch {
            name: body.login,
            id: body.user_id,
        })
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, Hash)]
pub enum AuthRequest {
    /// [](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#oauth-implicit-code-flow) requiring GET.
    ImplicitCode {
        client_id: String,
        redirect_uri: String,
        scope: Vec<String>,
        state: String,
        force_verify: bool,
    },
    /// [](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#oauth-authorization-code-flow) requiring GET.
    AuthorizationCode {
        client_id: String,
        redirect_uri: String,
        scope: Vec<String>,
        state: String,
        force_verify: bool,
    },
    /// [](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#oauth-client-credentials-flow) requiring POST.
    ClientCredentials {
        client_id: String,
        client_secret: String,
        scope: Vec<String>,
    },
}

impl AuthRequest {
    /// Creates a new [AuthRequest] with given client information and by fetching auth information
    /// from the environment ([ENV_TWITCH_AUTH] defaults to `"token"`, [ENV_TWITCH_SCOPES] defaults to [DEFAULT_SCOPES]).
    pub fn new(client_id: String, client_secret: Option<String>) -> Self {
        let auth_type = std::env::var(ENV_TWITCH_AUTH).unwrap_or_else(|arg| {
            warn!("Error fetching envvar: {}", arg);
            "token".to_string()
        });
        let scope = std::env::var(ENV_TWITCH_SCOPES)
            .map(|val| {
                let scopes: Vec<&str> = val.split(',').collect::<Vec<_>>();
                scopes.iter().map(|scope| scope.to_string()).collect()
            })
            .unwrap_or_else(|arg| {
                warn!("Error fetchng envvar: {}", arg);
                DEFAULT_SCOPES
                    .iter()
                    .map(|scope| scope.to_string())
                    .collect()
            });

        match auth_type.as_str() {
            "token" => AuthRequest::ImplicitCode {
                client_id,
                redirect_uri: REDIRECT_URI.to_string(),
                scope,
                state: rand_alphanumeric(30),
                force_verify: true,
            },
            "code" => AuthRequest::AuthorizationCode {
                client_id,
                redirect_uri: REDIRECT_URI.to_string(),
                scope,
                state: rand_alphanumeric(30),
                force_verify: true,
            },
            "client_credentials" => {
                let client_secret = client_secret.expect(
                    "client-secret required for client_credential authentication with twitch",
                );

                AuthRequest::ClientCredentials {
                    client_id,
                    client_secret,
                    scope,
                }
            }
            t => panic!("Unsupported twitch authentication Type: {}", t),
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
                force_verify,
            } => {
                let url = format!("https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri={}&response_type=token&scope={}&force_verify={}&state={}",
                                  client_id, redirect_uri, scope.join("%20"), force_verify, state);
                let url = Url::from_str(&url).expect("invalid auth url");
                write!(f, "{}", url)
            }
            AuthRequest::AuthorizationCode {
                client_id,
                redirect_uri,
                scope,
                state,
                force_verify,
            } => {
                let url = format!("https://id.twitch.tv/oauth2/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&force_verify={}&state={}", client_id, redirect_uri, scope.join("%20"), force_verify, state);
                let url = Url::from_str(&url).expect("invalid auth url");

                write!(f, "{}", url)
            }
            AuthRequest::ClientCredentials {
                client_id,
                client_secret,
                scope,
            } => {
                let url = format!("https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials&scope={}", client_id, client_secret, scope.join("%20"));
                let url = Url::from_str(&url).expect("invalid auth url");

                write!(f, "{}", url)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
struct TwitchValidation {
    client_id: String,
    login: String,
    user_id: String,
    scopes: Vec<String>,
    expires_in: i64,
}

#[cfg(test)]
mod tests {
    use crate::twitch_api::auth::{AuthRequest, DEFAULT_SCOPES, ENV_TWITCH_AUTH, REDIRECT_URI};

    #[test]
    fn test_format() {
        let auth = AuthRequest::ImplicitCode {
            client_id: "someclientid".to_string(),
            redirect_uri: "https://localhost:4334/".to_string(),
            scope: vec!["scope1".to_string(), "scope2".to_string()],
            force_verify: true,
            state: "abcdefghijklmnopqrstuvwxyz123456789".to_string(),
        };
        assert_eq!(auth.to_string(), "https://id.twitch.tv/oauth2/authorize?client_id=someclientid&redirect_uri=https://localhost:4334/&response_type=token&scope=scope1%20scope2&force_verify=true&state=abcdefghijklmnopqrstuvwxyz123456789")
    }

    #[test]
    fn test_auth_req_new_implicitcode() {
        std::env::set_var(ENV_TWITCH_AUTH, "token");
        let auth = AuthRequest::new("client_id".to_string(), None);

        match auth {
            AuthRequest::ImplicitCode {
                client_id,
                redirect_uri,
                scope,
                state: _,
                force_verify,
            } => {
                assert_eq!(client_id, "client_id");
                assert_eq!(redirect_uri, REDIRECT_URI);
                assert_eq!(scope, DEFAULT_SCOPES);
                assert_eq!(force_verify, true);
            }
            req => assert!(false, "Expected AuthReq::ImplicitCode but got: {:?}", req),
        }
    }

    #[test]
    fn test_auth_req_new_authcode() {
        std::env::set_var(ENV_TWITCH_AUTH, "code");
        let auth = AuthRequest::new("client_id".to_string(), None);

        match auth {
            AuthRequest::AuthorizationCode {
                client_id,
                redirect_uri,
                scope,
                state: _,
                force_verify,
            } => {
                assert_eq!(client_id, "client_id");
                assert_eq!(redirect_uri, REDIRECT_URI);
                assert_eq!(scope, DEFAULT_SCOPES);
                assert_eq!(force_verify, true);
            }
            req => assert!(false, "Expected AuthReq::ImplicitCode but got: {:?}", req),
        }
    }

    #[test]
    fn test_auth_req_new_clientcredentials() {
        std::env::set_var(ENV_TWITCH_AUTH, "client_credentials");
        let auth = AuthRequest::new("client_id".to_string(), Some("client_secret".to_string()));

        match auth {
            AuthRequest::ClientCredentials {
                client_id,
                client_secret,
                scope,
            } => {
                assert_eq!(client_id, "client_id");
                assert_eq!(client_secret, "client_secret");
                assert_eq!(scope, DEFAULT_SCOPES);
            }
            req => assert!(false, "Expected AuthReq::ClientCredentials but got: {:?}", req),
        }
    }
}
