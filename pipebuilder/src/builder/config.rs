use pipebuilder_common::BaseConfig;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BuilderConfig {
    pub repository_endpoint: String,
    pub workspace: String,
    pub restore_directory: String,
    pub log_directory: String,
    // reset directory when bootstrap, default as true if not provided
    pub reset: Option<bool>,
}

#[derive(Deserialize)]
pub struct Config {
    pub base: BaseConfig,
    pub builder: BuilderConfig,
}
