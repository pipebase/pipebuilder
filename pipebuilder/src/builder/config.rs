use pipebuilder_common::{NodeConfig, RegistryConfig};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub node: NodeConfig,
    // pub registry: RegistryConfig,
}
