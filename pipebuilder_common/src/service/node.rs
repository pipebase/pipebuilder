use crate::{
    grpc::node::{self, node_server::Node},
    Period, Register, DEFAULT_NODE_HEARTBEAT_PERIOD, ENV_PIPEBUILDER_EXTERNAL_ADDR,
    ENV_PIPEBUILDER_NODE_ID, REGISTER_KEY_PREFIX_API, REGISTER_KEY_PREFIX_BUILDER,
    REGISTER_KEY_PREFIX_REPOSITORY, REGISTER_KEY_PREFIX_SCHEDULER,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};
use tracing::error;

#[derive(Clone, Deserialize, Serialize)]
pub enum NodeRole {
    Api,
    Builder,
    Manifest,
    Scheduler,
}

pub fn role_prefix(role: NodeRole) -> &'static str {
    match role {
        NodeRole::Api => REGISTER_KEY_PREFIX_API,
        NodeRole::Builder => REGISTER_KEY_PREFIX_BUILDER,
        NodeRole::Manifest => REGISTER_KEY_PREFIX_REPOSITORY,
        NodeRole::Scheduler => REGISTER_KEY_PREFIX_SCHEDULER,
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum NodeStatus {
    Active,
    InActive,
}

impl From<u8> for NodeStatus {
    fn from(origin: u8) -> Self {
        match origin {
            0 => NodeStatus::Active,
            1 => NodeStatus::InActive,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Deserialize)]
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
    // node internal address
    pub internal_address: String,
    // node external address
    pub external_address: String,
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
    // node internal address
    internal_address: String,
    // node external address
    external_address: String,
    // node lease id for ownership of keys
    lease_id: i64,
    // node heartbeat period
    heartbeat_period: Duration,
    // node runtime status code
    status_code: Arc<AtomicU8>,
}

impl NodeService {
    pub fn new(config: NodeConfig, lease_id: i64) -> Self {
        let id = config.id;
        // environment variable overwrite configuration
        let id = std::env::var(ENV_PIPEBUILDER_NODE_ID).map_or(id, |id| id);
        let role = config.role;
        let internal_address = config.internal_address;
        let env_external_address = std::env::var(ENV_PIPEBUILDER_EXTERNAL_ADDR).ok();
        let config_external_address = config.external_address;
        // environment variable overwrite configuration
        let external_address = match (env_external_address, config_external_address) {
            (Some(external_address), _) => external_address,
            (_, Some(external_address)) => external_address,
            (None, None) => internal_address.clone(),
        };
        let heartbeat_period = config.heartbeat_period;
        let heartbeat_period = heartbeat_period.unwrap_or(DEFAULT_NODE_HEARTBEAT_PERIOD);
        NodeService {
            id,
            role,
            internal_address,
            external_address,
            lease_id,
            heartbeat_period: heartbeat_period.into(),
            status_code: Arc::new(AtomicU8::new(NodeStatus::Active as u8)),
        }
    }

    pub fn get_id(&self) -> String {
        self.id.to_owned()
    }

    pub fn get_internal_address(&self) -> String {
        self.internal_address.to_owned()
    }

    pub fn get_external_address(&self) -> String {
        self.external_address.to_owned()
    }

    pub fn get_status_code(&self) -> Arc<AtomicU8> {
        self.status_code.clone()
    }

    pub fn run(&self, mut register: Register) {
        let heartbeat_period = self.heartbeat_period.to_owned();
        let mut interval = tokio::time::interval(heartbeat_period);
        let id = self.id.to_owned();
        let role = self.role.to_owned();
        let internal_address = self.internal_address.to_owned();
        let external_address = self.external_address.to_owned();
        let status_code = self.status_code.clone();
        let lease_id = self.lease_id;
        let _ = tokio::spawn(async move {
            loop {
                interval.tick().await;
                // register or patch local node state
                let timestamp = Utc::now();
                let status_code = status_code.load(Ordering::Acquire);
                let state = NodeState {
                    id: id.clone(),
                    role: role.clone(),
                    internal_address: internal_address.clone(),
                    external_address: external_address.clone(),
                    status: status_code.into(),
                    timestamp,
                };
                match register
                    .put_node_state(role_prefix(role.clone()), &state, lease_id)
                    .await
                {
                    Ok(_) => continue,
                    Err(e) => {
                        error!(
                            "put node state error {:?}, node service stop state patching",
                            e
                        );
                        break;
                    }
                }
            }
        });
    }
}

#[tonic::async_trait]
impl Node for NodeService {
    async fn activate(
        &self,
        _request: tonic::Request<node::ActivateRequest>,
    ) -> Result<tonic::Response<node::ActivateResponse>, tonic::Status> {
        self.status_code
            .store(NodeStatus::Active as u8, Ordering::Release);
        Ok(tonic::Response::new(node::ActivateResponse {}))
    }

    async fn deactivate(
        &self,
        _request: tonic::Request<node::DeactivateRequest>,
    ) -> Result<tonic::Response<node::DeactivateResponse>, tonic::Status> {
        self.status_code
            .store(NodeStatus::InActive as u8, Ordering::Release);
        Ok(tonic::Response::new(node::DeactivateResponse {}))
    }

    async fn status(
        &self,
        _request: tonic::Request<node::StatusRequest>,
    ) -> Result<tonic::Response<node::StatusResponse>, tonic::Status> {
        let status: NodeStatus = self.status_code.load(Ordering::Acquire).into();
        let active = match status {
            NodeStatus::Active => true,
            NodeStatus::InActive => false,
        };
        Ok(tonic::Response::new(node::StatusResponse { active }))
    }
}
