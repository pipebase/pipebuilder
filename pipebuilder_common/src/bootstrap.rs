use crate::{
    BaseConfig, HealthService, LeaseConfig, LeaseService, NodeConfig, NodeService, Register,
    RegisterConfig, Result,
};
use tracing::info;

pub async fn build_register(config: RegisterConfig) -> Result<Register> {
    Register::new(config).await
}

pub fn build_lease_service(config: LeaseConfig, lease_id: i64) -> LeaseService {
    LeaseService::new(config, lease_id)
}

pub fn build_node_service(config: NodeConfig, lease_id: i64) -> NodeService {
    NodeService::new(config, lease_id)
}

pub async fn bootstrap(
    config: BaseConfig,
) -> Result<(Register, NodeService, HealthService, LeaseService)> {
    info!("bootstrap base service");
    // build register
    let mut register = build_register(config.register).await?;
    // lease grant
    let ttl = config.lease.ttl;
    let resp = register.lease_grant(ttl as i64).await?;
    let lease_id = resp.id();
    // build services
    let lease_svc = build_lease_service(config.lease, lease_id);
    let lease_id = lease_svc.get_lease_id();
    let node_svc = build_node_service(config.node, lease_id);
    lease_svc.run(register.clone());
    node_svc.run(register.clone());
    let health_svc = HealthService::default();
    // run svc
    Ok((register, node_svc, health_svc, lease_svc))
}
