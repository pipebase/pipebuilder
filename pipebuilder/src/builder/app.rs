mod bootstrap;
mod build;
mod config;

use bootstrap::bootstrap;
use config::Config;
use futures_util::FutureExt;
use pipebuilder_common::{
    grpc::{
        build::builder_server::BuilderServer, health::health_server::HealthServer,
        node::node_server::NodeServer,
    },
    init_tracing_subscriber, open_file, parse_config, Result, ENV_PIPEBUILDER_CONFIG_FILE,
};
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::{info, instrument};

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    init_tracing_subscriber();
    info!("read configuration ...");
    let file = open_file(std::env::var(ENV_PIPEBUILDER_CONFIG_FILE)?).await?;
    let config = parse_config::<Config>(file).await?;
    // bootstrap base service
    let (register, node_svc, health_svc, lease_svc, shutdown_rx) =
        pipebuilder_common::bootstrap(config.base).await?;
    let lease_id = lease_svc.get_lease_id();
    // bootstrap builder service
    let node_id = node_svc.get_id();
    let external_address = node_svc.get_external_address();
    let builder_svc = bootstrap(
        node_id.clone(),
        external_address,
        config.builder,
        lease_id,
        register,
    )
    .await?;
    // bootstrap server
    let internal_address = node_svc.get_internal_address();
    let addr: SocketAddr = internal_address.parse()?;
    info!(
        "run builder server {:?}, internal address {:?}...",
        node_id, internal_address
    );
    Server::builder()
        .add_service(HealthServer::new(health_svc))
        .add_service(BuilderServer::new(builder_svc))
        .add_service(NodeServer::new(node_svc))
        .serve_with_shutdown(addr, shutdown_rx.map(drop))
        .await?;
    info!("builder server {:?} exit ...", node_id);
    Ok(())
}
