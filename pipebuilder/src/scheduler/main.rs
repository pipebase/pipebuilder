mod bootstrap;
mod config;
mod schedule;

use config::Config;
use futures_util::FutureExt;
use pipebuilder_common::{
    bootstrap,
    grpc::{
        health::health_server::HealthServer, node::node_server::NodeServer,
        schedule::scheduler_server::SchedulerServer,
    },
    init_tracing_subscriber, open_file, parse_config, Result, ENV_PIPEBUILDER_CONFIG_FILE,
};
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::{info, instrument};

use crate::config::SchedulerConfig;

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    init_tracing_subscriber();
    info!("read configuration ...");
    let file = open_file(std::env::var(ENV_PIPEBUILDER_CONFIG_FILE)?).await?;
    let config = parse_config::<Config>(file).await?;
    let (register, node_svc, health_svc, _, shutdown_rx) = bootstrap(config.base).await?;
    let node_id = node_svc.get_id();
    let internal_address = node_svc.get_internal_address();
    let addr: SocketAddr = internal_address.parse()?;
    info!(
        node_id = node_id.as_str(),
        internal_address = internal_address.as_str(),
        "run scheduler server ..."
    );
    // bootstrap schedluer services
    let scheduler_svc = bootstrap::bootstrap(SchedulerConfig {}, register);
    Server::builder()
        .add_service(HealthServer::new(health_svc))
        .add_service(SchedulerServer::new(scheduler_svc))
        .add_service(NodeServer::new(node_svc))
        .serve_with_shutdown(addr, shutdown_rx.map(drop))
        .await?;
    info!(node_id = node_id.as_str(), "scheduler server exit ...");
    Ok(())
}
