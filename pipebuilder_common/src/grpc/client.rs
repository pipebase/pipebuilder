use crate::{
    errors::Result,
    grpc::{
        build::builder_client::BuilderClient, node::node_client::NodeClient,
        repository::repository_client::RepositoryClient,
        schedule::scheduler_client::SchedulerClient,
    },
};
use serde::Deserialize;
use std::fmt;
use tonic::transport::Channel;

#[derive(Deserialize)]
pub enum RpcProtocolType {
    Http,
    Https,
}

impl fmt::Display for RpcProtocolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpcProtocolType::Http => write!(f, "http"),
            RpcProtocolType::Https => write!(f, "https"),
        }
    }
}

#[derive(Deserialize)]
pub struct RpcClientConfig {
    pub protocol: RpcProtocolType,
    pub address: String,
}

impl RpcClientConfig {
    pub fn endpoint(&self) -> String {
        format!("{}://{}", self.protocol, self.address)
    }
}

#[derive(Default)]
pub struct BuilderClientBuilder<'a> {
    pub protocol: Option<RpcProtocolType>,
    pub address: Option<&'a str>,
}

impl<'a> BuilderClientBuilder<'a> {
    pub fn protocol(mut self, protocol: RpcProtocolType) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn address(mut self, address: &'a str) -> Self {
        self.address = Some(address);
        self
    }

    pub async fn connect(self) -> Result<BuilderClient<Channel>> {
        let client = BuilderClient::connect(format!(
            "{}://{}",
            self.protocol.expect("protocol undefined"),
            self.address.expect("address undefined")
        ))
        .await?;
        Ok(client)
    }
}

#[derive(Default)]
pub struct NodeClientBuilder<'a> {
    pub protocol: Option<RpcProtocolType>,
    pub address: Option<&'a str>,
}

impl<'a> NodeClientBuilder<'a> {
    pub fn protocol(mut self, protocol: RpcProtocolType) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn address(mut self, address: &'a str) -> Self {
        self.address = Some(address);
        self
    }

    pub async fn connect(self) -> Result<NodeClient<Channel>> {
        let client = NodeClient::connect(format!(
            "{}://{}",
            self.protocol.expect("protocol undefined"),
            self.address.expect("address undefined")
        ))
        .await?;
        Ok(client)
    }
}

#[derive(Default)]
pub struct RepositoryClientBuilder<'a> {
    pub protocol: Option<RpcProtocolType>,
    pub address: Option<&'a str>,
}

impl<'a> RepositoryClientBuilder<'a> {
    pub fn protocol(mut self, protocol: RpcProtocolType) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn address(mut self, address: &'a str) -> Self {
        self.address = Some(address);
        self
    }

    pub async fn connect(self) -> Result<RepositoryClient<Channel>> {
        let client = RepositoryClient::connect(format!(
            "{}://{}",
            self.protocol.expect("protocol undefined"),
            self.address.expect("address undefined")
        ))
        .await?;
        Ok(client)
    }
}

#[derive(Default)]
pub struct SchedulerClientBuilder<'a> {
    pub protocol: Option<RpcProtocolType>,
    pub address: Option<&'a str>,
}

impl<'a> SchedulerClientBuilder<'a> {
    pub fn protocol(mut self, protocol: RpcProtocolType) -> Self {
        self.protocol = Some(protocol);
        self
    }

    pub fn address(mut self, address: &'a str) -> Self {
        self.address = Some(address);
        self
    }

    pub async fn connect(self) -> Result<SchedulerClient<Channel>> {
        let client = SchedulerClient::connect(format!(
            "{}://{}",
            self.protocol.expect("protocol undefined"),
            self.address.expect("address undefined")
        ))
        .await?;
        Ok(client)
    }
}
