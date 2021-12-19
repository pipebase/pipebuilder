use pipebuilder_common::BaseConfig;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RepositoryConfig {
    pub manifest: String,
    pub app: String,
    pub catalog_schema: String,
    pub catalogs: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub base: BaseConfig,
    pub repository: RepositoryConfig,
}
