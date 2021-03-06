use crate::{
    grpc::node::{self, node_server::Node},
    Period, Register, Resource, ResourceType, DEFAULT_NODE_HEARTBEAT_PERIOD,
    ENV_PIPEBUILDER_EXTERNAL_ADDR, ENV_PIPEBUILDER_NODE_ID,
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
use tokio::sync::oneshot::Sender;
use tracing::{error, info};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum NodeRole {
    Api,
    Builder,
    Repository,
    Scheduler,
    Undefined,
}

impl ToString for NodeRole {
    fn to_string(&self) -> String {
        let role_text = match self {
            NodeRole::Api => "Api",
            NodeRole::Builder => "Builder",
            NodeRole::Repository => "Repository",
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
            "repository" | "Repository" => NodeRole::Repository,
            "scheduler" | "Scheduler" => NodeRole::Scheduler,
            _ => NodeRole::Undefined,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum NodeStatus {
    Active,
    InActive,
    Shutdown,
}

impl From<u8> for NodeStatus {
    fn from(origin: u8) -> Self {
        match origin {
            0 => NodeStatus::Active,
            1 => NodeStatus::InActive,
            2 => NodeStatus::Shutdown,
            _ => unreachable!(),
        }
    }
}

impl ToString for NodeStatus {
    fn to_string(&self) -> String {
        let status_text = match self {
            NodeStatus::Active => "Active",
            NodeStatus::InActive => "Inactive",
            NodeStatus::Shutdown => "Shutdown",
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
            NodeArch::UNKNOWN => String::from("unknown"),
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

impl Resource for NodeState {
    fn ty() -> ResourceType {
        ResourceType::Node
    }
}

impl NodeState {
    pub fn is_active(&self) -> bool {
        matches!(self.status, NodeStatus::Active)
    }

    pub fn accept_target_platform(&self, target_platform: &str) -> bool {
        match target_platform {
            "aarch64-unknown-linux-gnu" => {
                matches!(self.arch, NodeArch::AARCH64) && matches!(self.os, NodeOS::LINUX)
            }
            "x86_64-apple-darwin" => {
                matches!(self.arch, NodeArch::X86_64) && matches!(self.os, NodeOS::MACOS)
            }
            "x86_64-unknown-linux-gnu" => {
                matches!(self.arch, NodeArch::X86_64) && matches!(self.os, NodeOS::LINUX)
            }
            _ => false,
        }
    }

    pub fn get_support_target_platform(&self) -> Option<String> {
        match (&self.arch, &self.os) {
            (NodeArch::AARCH64, NodeOS::LINUX) => Some(String::from("aarch64-unknown-linux-gnu")),
            (NodeArch::X86_64, NodeOS::MACOS) => Some(String::from("x86_64-apple-darwin")),
            (NodeArch::X86_64, NodeOS::LINUX) => Some(String::from("x86_64-unknown-linux-gnu")),
            (_, _) => None,
        }
    }
}

#[derive(Clone)]
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

    pub fn run(&self, mut register: Register, shutdown_tx: Sender<()>) {
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
                let status: NodeStatus = status_code.into();
                let state = NodeState {
                    role: role.clone(),
                    arch: arch.clone(),
                    os: os.clone(),
                    internal_address: internal_address.clone(),
                    external_address: external_address.clone(),
                    status: status.clone(),
                    timestamp,
                };
                match register
                    .put_resource::<NodeState>(None, id.as_str(), None, state, lease_id)
                    .await
                {
                    Ok(_) => (),
                    Err(e) => {
                        error!(
                            "put node state error {:?}, node service stop state patching",
                            e
                        );
                        break;
                    }
                };
                if matches!(status, NodeStatus::Shutdown) {
                    break;
                }
            }
            // shutdown server either we received ctrl signal or patch node state failed
            info!(
                role = role.to_string().as_str(),
                id = id.as_str(),
                "shutdown ..."
            );
            shutdown_tx.send(()).unwrap();
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
        let active = matches!(status, NodeStatus::Active);
        Ok(tonic::Response::new(node::StatusResponse { active }))
    }

    async fn shutdown(
        &self,
        _request: tonic::Request<node::ShutdownRequest>,
    ) -> Result<tonic::Response<node::ShutdownResponse>, tonic::Status> {
        self.status_code
            .store(NodeStatus::Shutdown as u8, Ordering::Release);
        Ok(tonic::Response::new(node::ShutdownResponse {}))
    }
}
