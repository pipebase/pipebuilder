use pipebuilder_common::{LeaseConfig, NodeConfig, RegisterConfig};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub node: NodeConfig,
    pub register: RegisterConfig,
    pub lease: LeaseConfig,
}
