use crate::Result;
use reqwest::{
    header::{HeaderMap, HeaderName},
    Body, Client, Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Deserialize)]
pub struct BasicAuth {
    pub username: String,
    pub password: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct ApiClientConfig {
    // api endpoint
    pub endpoint: String,
    pub basic_auth: Option<BasicAuth>,
    pub bearer_auth_token: Option<String>,
    pub headers: HashMap<String, String>,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        ApiClientConfig {
            endpoint: String::from("127.0.0.1:16000"),
            basic_auth: None,
            bearer_auth_token: None,
            headers: HashMap::new(),
        }
    }
}

pub struct ApiClient {
    client: Client,
    endpoint: String,
    basic_auth: Option<BasicAuth>,
    bearer_auth_token: Option<String>,
    headers: HeaderMap,
}

impl From<ApiClientConfig> for ApiClient {
    fn from(config: ApiClientConfig) -> Self {
        let endpoint = config.endpoint;
        let basic_auth = config.basic_auth;
        let bearer_auth_token = config.bearer_auth_token;
        let headers = config.headers;
        let mut hmap = HeaderMap::new();
        for (name, value) in &headers {
            hmap.insert::<HeaderName>(
                name.parse()
                    .unwrap_or_else(|_| panic!("invalid header name '{}'", name)),
                value
                    .parse()
                    .unwrap_or_else(|_| panic!("invalid header value '{}'", value)),
            );
        }
        ApiClient {
            client: Client::new(),
            endpoint,
            basic_auth,
            bearer_auth_token,
            headers: hmap,
        }
    }
}

impl ApiClient {
    fn get_url(&self, path: &str) -> String {
        format!("{}{}", self.endpoint, path)
    }

    pub async fn post<B>(&self, path: &str, body: B) -> Result<Response>
    where
        B: Into<Body>,
    {
        let req = self
            .client
            .post(self.get_url(path))
            .headers(self.headers.to_owned());
        let req = match self.basic_auth {
            Some(ref basic_auth) => {
                req.basic_auth(&basic_auth.username, basic_auth.password.as_ref())
            }
            None => req,
        };
        let req = match self.bearer_auth_token {
            Some(ref token) => req.bearer_auth(token),
            None => req,
        };
        let resp = req.body(body).send().await?;
        Ok(resp)
    }

    pub async fn query<Q>(&self, path: &str, query: Option<Q>) -> Result<Response>
    where
        Q: Serialize,
    {
        let req = self
            .client
            .get(self.get_url(path))
            .headers(self.headers.to_owned());
        let req = match query {
            Some(ref query) => req.query(query),
            None => req,
        };
        let req = match self.basic_auth {
            Some(ref basic_auth) => {
                req.basic_auth(&basic_auth.username, basic_auth.password.as_ref())
            }
            None => req,
        };
        let req = match self.bearer_auth_token {
            Some(ref token) => req.bearer_auth(token),
            None => req,
        };
        let resp = req.send().await?;
        Ok(resp)
    }
}
