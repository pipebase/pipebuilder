mod bootstrap;
mod config;
mod repository;

use bootstrap::bootstrap;
use config::Config;
use pipebuilder_common::{
    grpc::{health::health_server::HealthServer, repository::repository_server::RepositoryServer},
    open_file, parse_config, Result, ENV_PIPEBUILDER_CONFIG_FILE,
};
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::{info, instrument};

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("read configuration ...");
    let file = open_file(std::env::var(ENV_PIPEBUILDER_CONFIG_FILE)?)?;
    let config = parse_config::<Config>(file)?;
    // bootstrap base svc
    let (register, node_svc, health_svc, lease_svc) =
        pipebuilder_common::bootstrap(config.base).await?;
    // bootstrap repository svc
    let lease_id = lease_svc.get_lease_id();
    let repository_svc = bootstrap(config.repository, register, lease_id);
    // bootstrap server
    let node_id = node_svc.get_id().to_owned();
    let internal_address = node_svc.get_internal_address().to_owned();
    let addr: SocketAddr = internal_address.parse()?;
    info!(
        "run repository server {:?}, internal address {:?}...",
        node_id, internal_address
    );
    Server::builder()
        .add_service(HealthServer::new(health_svc))
        .add_service(RepositoryServer::new(repository_svc))
        .serve(addr)
        .await?;
    info!("repository server {:?} exit ...", node_id);
    Ok(())
}
