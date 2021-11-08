use crate::{
    grpc::node::{self, node_server::Node},
    Period, Register, DEFAULT_NODE_HEARTBEAT_PERIOD, ENV_PIPEBUILDER_EXTERNAL_ADDR,
    ENV_PIPEBUILDER_NODE_ID, RESOURCE_NODE_API, RESOURCE_NODE_BUILDER, RESOURCE_NODE_REPOSITORY,
    RESOURCE_NODE_SCHEDULER,
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
    Undefined,
}

impl ToString for NodeRole {
    fn to_string(&self) -> String {
        let role_text = match self {
            NodeRole::Api => "Api",
            NodeRole::Builder => "Builder",
            NodeRole::Manifest => "Manifest",
            NodeRole::Scheduler => "Scheduler",
            NodeRole::Undefined => unreachable!(),
        };
        String::from(role_text)
    }
}

impl From<&str> for NodeRole {
    fn from(text: &str) -> Self {
        match text {
            "api" | "Api" => NodeRole::Api,
            "builder" | "Builder" => NodeRole::Builder,
            "manifest" | "Manifest" => NodeRole::Manifest,
            "scheduler" | "Scheduler" => NodeRole::Scheduler,
            _ => NodeRole::Undefined,
        }
    }
}

pub fn node_role_prefix(role: &NodeRole) -> &'static str {
    match role {
        NodeRole::Api => RESOURCE_NODE_API,
        NodeRole::Builder => RESOURCE_NODE_BUILDER,
        NodeRole::Manifest => RESOURCE_NODE_REPOSITORY,
        NodeRole::Scheduler => RESOURCE_NODE_SCHEDULER,
        NodeRole::Undefined => unreachable!(),
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

impl ToString for NodeStatus {
    fn to_string(&self) -> String {
        let status_text = match self {
            NodeStatus::Active => "Active",
            NodeStatus::InActive => "Inactive",
        };
        String::from(status_text)
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

#[derive(Clone, Deserialize, Serialize)]
pub enum NodeArch {
    X86_64,
    AARCH64,
    UNKNOWN,
}

impl From<&str> for NodeArch {
    fn from(arch: &str) -> Self {
        match arch {
            "x86_64" => NodeArch::X86_64,
            "aarch64" => NodeArch::AARCH64,
            _ => NodeArch::UNKNOWN,
        }
    }
}

impl ToString for NodeArch {
    fn to_string(&self) -> String {
        match self {
            NodeArch::X86_64 => String::from("x86_64"),
            NodeArch::AARCH64 => String::from("aarch64"),
            &NodeArch::UNKNOWN => String::from("unknown"),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum NodeOS {
    LINUX,
    MACOS,
    UNKNOWN,
}

impl From<&str> for NodeOS {
    fn from(os: &str) -> Self {
        match os {
            "linux" => NodeOS::LINUX,
            "macos" => NodeOS::MACOS,
            _ => NodeOS::UNKNOWN,
        }
    }
}

impl ToString for NodeOS {
    fn to_string(&self) -> String {
        match self {
            NodeOS::LINUX => String::from("linux"),
            NodeOS::MACOS => String::from("macos"),
            NodeOS::UNKNOWN => String::from("unknown"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NodeState {
    // node id
    pub id: String,
    // node role
    pub role: NodeRole,
    // node arch
    pub arch: NodeArch,
    // node os
    pub os: NodeOS,
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
    // node arch
    arch: NodeArch,
    // node os
    os: NodeOS,
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
        let arch: NodeArch = std::env::consts::ARCH.into();
        let os: NodeOS = std::env::consts::OS.into();
        NodeService {
            id,
            role,
            arch,
            os,
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
        let arch = self.arch.to_owned();
        let os = self.os.to_owned();
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
                    arch: arch.clone(),
                    os: os.clone(),
                    internal_address: internal_address.clone(),
                    external_address: external_address.clone(),
                    status: status_code.into(),
                    timestamp,
                };
                match register.put_node_state(&role, &state, lease_id).await {
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
