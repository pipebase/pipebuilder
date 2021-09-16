use crate::{
    Period, Register, Result, DEFAULT_NODE_HEARTBEAT_PERIOD, ENV_PIPEBUILDER_EXTERNAL_ADDR,
    ENV_PIPEBUILDER_NODE_ID, REGISTER_KEY_BUILDER_NODE_ID_PREFIX,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Deserialize, Serialize)]
pub enum NodeRole {
    Api,
    Builder,
    Scheduler,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum NodeStatus {
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

#[derive(Serialize, Deserialize)]
pub struct NodeState {
    // node id
    pub id: String,
    // node role
    pub role: NodeRole,
    // node external address
    pub address: String,
    // status
    pub status: NodeStatus,
    // timestamp
    pub timestamp: DateTime<Utc>,
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
    // node runtime status
    status: NodeStatus,
    // etcd register
    register: Register,
}

impl NodeService {
    pub fn new(config: NodeConfig, register: Register, lease_id: i64) -> Self {
        let id = config.id;
        // environment variable overwrite configuration
        let id = std::env::var(ENV_PIPEBUILDER_NODE_ID).map_or(id, |id| id);
        let role = config.role;
        let env_external_addr = std::env::var(ENV_PIPEBUILDER_EXTERNAL_ADDR).ok();
        let config_external_addr = config.external_address;
        // environment variable overwrite configuration
        let address = match (env_external_addr, config_external_addr) {
            (Some(external_addr), _) => external_addr,
            (_, Some(external_addr)) => external_addr,
            (None, None) => config.internal_address,
        };
        let heartbeat_period = config.heartbeat_period;
        let heartbeat_period = heartbeat_period.unwrap_or(DEFAULT_NODE_HEARTBEAT_PERIOD);
        NodeService {
            id,
            role,
            address,
            lease_id,
            heartbeat_period: heartbeat_period.into(),
            status: NodeStatus::Active,
            register,
        }
    }

    fn get_node_state(&self) -> NodeState {
        let id = self.id.to_owned();
        let role = self.role.to_owned();
        let address = self.address.to_owned();
        let status = self.status.to_owned();
        let timestamp = Utc::now();
        NodeState {
            id,
            role,
            address,
            status,
            timestamp,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let heartbeat_period = self.heartbeat_period.to_owned();
        let mut interval = tokio::time::interval(heartbeat_period);
        loop {
            interval.tick().await;
            // register or patch local node state
            let state = self.get_node_state();
            self.register
                .put_node_state(REGISTER_KEY_BUILDER_NODE_ID_PREFIX, &state, self.lease_id)
                .await?;
        }
    }
}
