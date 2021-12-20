use pipebuilder_common::{grpc::client::RpcClientConfig, BaseConfig};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BuilderConfig {
    pub repository_client: RpcClientConfig,
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
