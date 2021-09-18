use crate::{LeaseConfig, NodeConfig, RegisterConfig};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BaseConfig {
    pub node: NodeConfig,
    pub register: RegisterConfig,
    pub lease: LeaseConfig,
}
