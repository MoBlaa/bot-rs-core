use reqwest::{Client, RequestBuilder};

pub mod auth;
pub mod follows;
pub mod users;

pub trait Req: Sized {
    fn url(&self) -> String;
}

pub trait ReqV5: Req {
    fn into_builder(self, client: &Client, client_id: String) -> RequestBuilder {
        client
            .get(&self.url())
            .header(reqwest::header::ACCEPT, "application/vnd.twitchtv.v5+json")
            .header("client-id", client_id)
    }
}

pub trait ReqNew: Req {
    fn authorization(&self) -> String;
    fn into_builder(self, client: &Client, client_id: String) -> RequestBuilder {
        client
            .get(&self.url())
            .header(reqwest::header::AUTHORIZATION, self.authorization())
            .header("client-id", client_id)
    }
}
