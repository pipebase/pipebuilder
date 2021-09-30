mod api;
mod bootstrap;
mod config;

use bootstrap::bootstrap;
use config::Config;
use pipebuilder_common::{open_file, parse_config, Result, ENV_PIPEBUILDER_CONFIG_FILE};
use std::net::SocketAddr;
use tracing::{info, instrument};

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("read configuration ...");
    let file = open_file(std::env::var(ENV_PIPEBUILDER_CONFIG_FILE)?)?;
    let config = parse_config::<Config>(file)?;
    // bootstrap base service
    let (register, node_svc, _, lease_svc) = pipebuilder_common::bootstrap(config.base).await?;
    // bootstrap api service
    let lease_id = lease_svc.get_lease_id();
    let api = bootstrap(config.api, register, lease_id).await?;
    // bootstrap server
    let node_id = node_svc.get_id();
    let internal_address = node_svc.get_internal_address();
    let addr: SocketAddr = internal_address.parse()?;
    info!(
        "run api server {:?}, internal address {:?}...",
        node_id, internal_address
    );
    warp::serve(api).run(addr).await;
    Ok(())
}
