use std::net::SocketAddr;

use tonic::transport::Server;

mod bootstrap;
mod config;

use bootstrap::bootstrap;
use config::Config;
use pipebuilder_common::{open_file, parse_config, Result, ENV_PIPEBUILDER_CONFIG_FILE};
use tracing::{info, instrument};

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("read configuration ...");
    let file = open_file(std::env::var(ENV_PIPEBUILDER_CONFIG_FILE)?)?;
    let config = parse_config::<Config>(file)?;
    let (node_svc, health_svc) = bootstrap(config).await?;

    let node_id = node_svc.get_id().to_owned();
    let internal_address = node_svc.get_internal_address().to_owned();
    let addr: SocketAddr = internal_address.parse()?;

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
