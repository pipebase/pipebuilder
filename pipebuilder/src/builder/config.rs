use pipebuilder_common::BaseConfig;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BuilderConfig {
    pub manifest_endpoint: String,
    pub workspace: String,
    pub restore_directory: String,
    pub log_directory: String,
    pub publish_directory: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub base: BaseConfig,
    pub builder: BuilderConfig,
}
