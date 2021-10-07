use pipebuilder_common::api::client::ApiClientConfig;
use serde::Deserialize;

#[derive(Clone, Default, Deserialize)]
pub struct Config {
    // api client config
    pub api: ApiClientConfig,
}
