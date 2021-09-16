use pipebuilder_common::{NodeConfig, RegisterConfig};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub node: NodeConfig,
    pub register: RegisterConfig,
}
