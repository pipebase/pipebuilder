mod admin;
mod api;
mod app;
mod bootstrap;
mod catalogs;
mod config;
mod utils;

use bootstrap::bootstrap;
use config::MockConfig;
use pipebuilder_common::{
    init_tracing_subscriber, open_file, parse_config, Result, ENV_PIPEBUILDER_CONFIG_FILE,
};
use std::net::SocketAddr;
use tracing::{error, info, instrument};

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    init_tracing_subscriber();
    info!("read configuration ...");
    let file = open_file(std::env::var(ENV_PIPEBUILDER_CONFIG_FILE)?).await?;
    let config = parse_config::<MockConfig>(file).await?;
    let (api, mut shutdown_rx) = bootstrap(config.repository)?;
    let address = config.address;
    let addr: SocketAddr = address.parse()?;
    info!(address = address.as_str(), "run mock server ...");
    let (_, server) = warp::serve(api).bind_with_graceful_shutdown(addr, async move {
        match shutdown_rx.recv().await {
            Some(_) => info!("mock server shutdown ..."),
            None => error!("sender drop, shutdown ..."),
        }
    });
    server.await;
    Ok(())
}
