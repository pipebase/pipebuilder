use crate::{
    Period, Result, DEFAULT_NODE_HEARTBEAT_PERIOD, ENV_PIPEBUILDER_EXTERNAL_ADDR,
    ENV_PIPEBUILDER_NODE_ID,
};
use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, Deserialize)]
pub enum NodeRole {
    Api,
    Builder,
    Scheduler,
}

#[derive(Clone, Deserialize)]
pub enum NodeState {
    Active,
    InActive,
}

#[derive(Deserialize)]
pub struct NodeConfig {
    pub id: String,
    pub role: NodeRole,
    pub internal_address: String,
    pub external_address: Option<String>,
    pub heartbeat_period: Option<Period>,
}

pub struct NodeService {
    // node id
    id: String,
    // node role
    role: NodeRole,
    // node external address
    address: String,
    // node lease id for ownership of keys
    lease_id: i64,
    // node heartbeat period
    heartbeat_period: Duration,
}

impl NodeService {
    pub fn new(config: &NodeConfig, lease_id: i64) -> Self {
        let id = config.id.to_owned();
        // environment variable overwrite configuration
        let id = std::env::var(ENV_PIPEBUILDER_NODE_ID).map_or(id, |id| id);
        let role = config.role.to_owned();
        let env_external_addr = std::env::var(ENV_PIPEBUILDER_EXTERNAL_ADDR).ok();
        let config_external_addr = config.external_address.to_owned();
        // environment variable overwrite configuration
        let address = match (env_external_addr, config_external_addr) {
            (Some(external_addr), _) => external_addr,
            (_, Some(external_addr)) => external_addr,
            (None, None) => config.internal_address.to_owned(),
        };
        let heartbeat_period = config.heartbeat_period.to_owned();
        let heartbeat_period = heartbeat_period.unwrap_or(DEFAULT_NODE_HEARTBEAT_PERIOD);
        NodeService {
            id,
            role,
            address,
            lease_id,
            heartbeat_period: heartbeat_period.into(),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let heartbeat_period = self.heartbeat_period.to_owned();
        let mut interval = tokio::time::interval(heartbeat_period);
        loop {
            interval.tick().await;
            // patch local node state
        }
    }
}
