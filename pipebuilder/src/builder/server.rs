use std::net::SocketAddr;

use tonic::transport::Server;
mod config;

use config::Config;
use pipebuilder_common::{
    health::health_server::HealthServer, open_file, parse_config, Error, HealthService,
    ENV_PIPEBUILDER_CONFIG_FILE,
};
use tracing::{info, instrument};

#[tokio::main]
#[instrument]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    info!("read configuration ...");
    let file = open_file(std::env::var(ENV_PIPEBUILDER_CONFIG_FILE)?)?;
    let config = parse_config::<Config>(file)?;
    let node = &config.node;
    let node_id = &node.id;
    let internal_address = &node.internal_address;
    let addr: SocketAddr = internal_address.parse()?;
    let health_svc = HealthServer::new(HealthService::default());
    info!(
        "run builder server {:?}, internal address {:?}...",
        node_id, internal_address
    );
    Server::builder()
        .add_service(health_svc)
        .serve(addr)
        .await?;
    info!("builder server {:?} exit ...", node_id);
    Ok(())
}
