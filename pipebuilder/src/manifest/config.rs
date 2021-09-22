use pipebuilder_common::BaseConfig;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ManifestConfig {
    pub repository: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub base: BaseConfig,
    pub manifest: ManifestConfig,
}
