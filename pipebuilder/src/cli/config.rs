use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct BasicAuth {
    pub username: String,
    pub password: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    // api endpoint
    pub endpoint: String,
    pub basic_auth: Option<BasicAuth>,
    pub bearer_auth_token: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            endpoint: String::from("127.0.0.1:16000"),
            basic_auth: None,
            bearer_auth_token: None,
        }
    }
}
