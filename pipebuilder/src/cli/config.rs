use pipebuilder_common::{api::client::ApiClientConfig, open_file, parse_config, Result};
use serde::Deserialize;

const DEFAULT_CONFIG_FILE: &str = "~/.pb/config";

#[derive(Clone, Default, Deserialize)]
pub struct Config {
    // api client config
    pub api: ApiClientConfig,
}

impl Config {
    pub(crate) async fn parse(path: Option<&str>) -> Result<Self> {
        let path = path.unwrap_or(DEFAULT_CONFIG_FILE);
        let file = open_file(path).await?;
        parse_config(file).await
    }

    pub(crate) async fn parse_or_default(path: Option<&str>) -> Self {
        Self::parse(path).await.unwrap_or_default()
    }
}
