use pipebuilder_common::BaseConfig;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RepositoryConfig {
    pub manifest: String,
    pub app: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub base: BaseConfig,
    pub repository: RepositoryConfig,
}
