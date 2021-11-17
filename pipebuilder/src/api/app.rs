mod api;
mod bootstrap;
mod config;

use bootstrap::bootstrap;
use config::Config;
use pipebuilder_common::{open_file, parse_config, Result, ENV_PIPEBUILDER_CONFIG_FILE};
use std::net::SocketAddr;
use tracing::{error, info, instrument};

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("read configuration ...");
    let file = open_file(std::env::var(ENV_PIPEBUILDER_CONFIG_FILE)?).await?;
    let config = parse_config::<Config>(file).await?;
    // bootstrap base service
    let (register, node_svc, _, lease_svc, shutdown_rx) =
        pipebuilder_common::bootstrap(config.base).await?;
    // bootstrap api service / server
    let lease_id = lease_svc.get_lease_id();
    let node_id = node_svc.get_id();
    let internal_address = node_svc.get_internal_address();
    let addr: SocketAddr = internal_address.parse()?;
    info!(
        "run api server {:?}, internal address {:?}...",
        node_id, internal_address
    );
    let api = bootstrap(config.api, register, lease_id, node_svc).await?;
    let (_, server) = warp::serve(api).bind_with_graceful_shutdown(addr, async move {
        match shutdown_rx.await {
            Ok(_) => info!("shutdown ..."),
            Err(_) => error!("sender(node service) drop, shutdown ..."),
        }
    });
    server.await;
    Ok(())
}
