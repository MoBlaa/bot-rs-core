use std::fmt::{Display, Formatter};
use core::fmt;
use url::Url;
use std::str::FromStr;
use crate::utils::rand_alphanumeric;
use std::sync::{Mutex, Arc, Condvar};
use std::thread;
use rocket::response::content;
use rocket::State;
use rocket::config::Environment;

/// Configuring which authentication method should be used.
/// "token" = OAuth Implicit Code Flow,
/// "code" = OAuth Authorization Code Flow,
/// "client_credentials" = OAuth Client Credentials Flow
pub const ENV_TWITCH_AUTH: &str = "BRS_TWITCH_AUTH";
pub const TWITCH_CLIENT_ID: &str = env!("BRS_TWITCH_CLIENT_ID");
pub const TWITCH_CLIENT_SECRET: Option<&str> = option_env!("BRS_TWITCH_CLIENT_SECRET");

/// Optional Comma separated List of scopes defined at [](https://dev.twitch.tv/docs/authentication/#scopes). Defaults to: `["channel:moderate","chat:edit","chat:read","user:edit:follows","user_follows_edit"]`
pub const ENV_TWITCH_SCOPES: &str = "BRS_TWITCH_SCOPES";
const TWITCH_OAUTH_HANDLER_SCRIPT: &str = include_str!("twitch_oauth.html");

static REDIRECT_URI: &str = "http://localhost:4334/";
static DEFAULT_SCOPES: [&str;6] = ["channel:moderate","chat:edit","chat:read","user:edit:follows","user_follows_edit", "user:edit"];

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum UserInfo {
    Twitch {
        login: String,
        user_id: String
    },
    None
}

impl UserInfo {
    pub fn name(&self) -> String {
        match self {
            UserInfo::Twitch {login, ..} => login.clone(),
            UserInfo::None => String::new()
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum Credentials {
    OAuthToken {
        token: String
    },
    None
}

impl Credentials {
    pub fn oauth(token: String) -> Credentials {
        Credentials::OAuthToken {token}
    }

    pub fn to_header(&self) -> String {
        match self {
            Credentials::OAuthToken {token} => format!("OAuth {}", token),
            Credentials::None => panic!("tried to convert Credentials::NONE to header"),
        }
    }
}

impl Display for Credentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Credentials::OAuthToken {token} => write!(f, "oauth:{}", token),
            Credentials::None => write!(f, "NONE")
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub struct Authentication {
    pub credentials: Credentials,
    pub user_info: UserInfo
}

type AuthMutex = Arc<(Mutex<Option<Authentication>>, Condvar)>;

/// Endpoint serving the html to read the OAuth fragment generated.
#[get("/")]
fn index() -> content::Html<&'static str> {
    content::Html(TWITCH_OAUTH_HANDLER_SCRIPT)
}

/// Endpoint to actually register.
#[post("/auth?<access_token>&<state>")]
fn auth_get(auth_req: State<AuthRequest>, auth: State<AuthMutex>, access_token: String, state: Option<String>) -> String {
    let nonce = match auth_req.inner() {
        AuthRequest::ImplicitCode { ref state, .. } => state,
        AuthRequest::AuthorizationCode { ref state, .. } => state,
        _ => ""
    };
    let state = state.expect("missing state in OAuth redirect");
    if !nonce.is_empty() && state != nonce {
        panic!("state doesn't match. Expected={}, Actual={}", nonce, state);
    }

    let (lock, cvar) = auth.as_ref();
    let mut token = lock.lock().unwrap();

    *token = Some(Authentication {
        credentials: Credentials::OAuthToken { token: access_token },
        user_info: UserInfo::None
    });
    cvar.notify_all();

    "Successfully obtained access token! You can close this window now..".to_string()
}

impl Authentication {
    pub fn twitch() -> Authentication {
        let req = AuthRequest::default();
        info!("For authentication please grant Nemabot access to the Bots Twitch account at: '{}'", req.to_string());
        let auth_lock: AuthMutex = Arc::new((Mutex::new(None), Condvar::new()));

        let r_auth_lock = Arc::clone(&auth_lock);
        thread::spawn(move || {
            let cfg = rocket::Config::build(Environment::active().expect("missing rocket environment"))
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
        while auth.as_ref().map(|auth| &auth.credentials).is_none() {
            auth = cvar.wait(auth).unwrap();
        }
        let mut auth = auth.take().unwrap();

        auth.user_info = auth.validate_token();

        auth
    }

    fn validate_token(&self) -> UserInfo {
        let client = reqwest::blocking::Client::new();
        let response: reqwest::blocking::Response = client.get("https://id.twitch.tv/oauth2/validate")
            .header("Authorization", self.credentials.to_header())
            .send()
            .expect("validation request failed");
        if !response.status().is_success() {
            panic!("validation request failed: {}", response.status());
        }
        let body: String = response.text()
            .expect("invalid response body");
        let body: TwitchValidation = serde_json::from_str(&body).unwrap();
        if body.client_id != TWITCH_CLIENT_ID {
            panic!("Client-ID doesn't match the one used for auth. Expected: {}, Actual: {}", TWITCH_CLIENT_ID, body.client_id);
        }
        UserInfo::Twitch {
            login: body.login,
            user_id: body.user_id,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
pub enum AuthRequest {
    /// [](https://dev.twitch.tv/docs/authentication/getting-tokens-oauth#oauth-implicit-code-flow) requiring GET.
    ImplicitCode {
        client_id: String,
        redirect_uri: String,
        scope: Vec<String>,
        state: String,
        force_verify: bool
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
    }
}

impl Default for AuthRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct TwitchValidation {
    client_id: String,
    login: String,
    user_id: String,
    scopes: Vec<String>,
    expires_in: usize,
}

impl AuthRequest {
    pub fn new() -> AuthRequest {
        let auth_type = std::env::var(ENV_TWITCH_AUTH)
            .unwrap_or_else(|arg| {
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
                DEFAULT_SCOPES.iter().map(|scope| scope.to_string()).collect()
            });

        match auth_type.as_str() {
            "token" => AuthRequest::ImplicitCode {
                client_id: TWITCH_CLIENT_ID.to_string(),
                redirect_uri: REDIRECT_URI.to_string(),
                scope,
                state: rand_alphanumeric(30),
                force_verify: false
            },
            "code" => AuthRequest::AuthorizationCode {
                client_id: TWITCH_CLIENT_ID.to_string(),
                redirect_uri: REDIRECT_URI.to_string(),
                scope,
                state: rand_alphanumeric(30),
                force_verify: false
            },
            "client_credentials" => {
                let client_secret = TWITCH_CLIENT_SECRET
                    .expect("client-secret required for client_credential authentication with twitch");

                AuthRequest::ClientCredentials {
                    client_id: TWITCH_CLIENT_ID.to_string(),
                    client_secret: client_secret.to_string(),
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
            redirect_uri: "https://localhost:4334/".to_string(),
            scope: vec!["scope1".to_string(), "scope2".to_string()],
            force_verify: true,
            state: "abcdefghijklmnopqrstuvwxyz123456789".to_string()
        };
        assert_eq!(auth.to_string(), "https://id.twitch.tv/oauth2/authorize?client_id=someclientid&redirect_uri=https://localhost:4334/&response_type=token&scope=scope1%20scope2&force_verify=true&state=abcdefghijklmnopqrstuvwxyz123456789")
    }
}
