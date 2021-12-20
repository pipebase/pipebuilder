use pipebuilder_common::{grpc::client::RpcClientConfig, BaseConfig};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RpcClientConfigs {
    pub repository: RpcClientConfig,
    pub scheduler: RpcClientConfig,
}

#[derive(Deserialize)]
pub struct ApiConfig {
    pub clients: RpcClientConfigs,
}

#[derive(Deserialize)]
pub struct Config {
    pub base: BaseConfig,
    pub api: ApiConfig,
}
