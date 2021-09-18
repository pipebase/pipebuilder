use crate::Config;
use pipebuilder_common::{
    health::health_server::HealthServer, HealthService, LeaseConfig, LeaseService, NodeConfig,
    NodeService, Register, RegisterConfig, Result,
};

pub async fn build_register(config: RegisterConfig) -> Result<Register> {
    config.into_register().await
}

pub fn build_lease_service(config: LeaseConfig, lease_id: i64) -> LeaseService {
    LeaseService::new(config, lease_id)
}

pub fn build_node_service(config: NodeConfig, lease_id: i64) -> NodeService {
    NodeService::new(config, lease_id)
}

pub async fn bootstrap(config: Config) -> Result<(NodeService, HealthServer<HealthService>)> {
    let base_config = config.base;
    // build register
    let mut register = build_register(base_config.register).await?;
    // lease grant
    let ttl = base_config.lease.ttl;
    let resp = register.lease_grant(ttl as i64).await?;
    let lease_id = resp.id();
    // build services
    let lease_svc = build_lease_service(base_config.lease, lease_id);
    let lease_id = lease_svc.get_lease_id();
    let node_svc = build_node_service(base_config.node, lease_id);
    lease_svc.run(register.clone());
    node_svc.run(register.clone());
    let health_svc = HealthServer::new(HealthService::default());
    // run svc
    Ok((node_svc, health_svc))
}
