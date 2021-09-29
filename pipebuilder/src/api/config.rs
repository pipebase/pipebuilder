use pipebuilder_common::BaseConfig;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GrpcClientConfig {
    pub endpoint: String,
}

#[derive(Deserialize)]
pub struct GrpcClientConfigs {
    pub manifest: GrpcClientConfig,
    pub scheduler: GrpcClientConfig,
}

#[derive(Deserialize)]
pub struct ApiConfig {
    pub clients: GrpcClientConfigs,
}

#[derive(Deserialize)]
pub struct Config {
    pub base: BaseConfig,
    pub api: ApiConfig,
}
