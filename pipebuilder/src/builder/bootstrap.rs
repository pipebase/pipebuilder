use crate::{
    build::{BuildManager, BuilderService},
    config::BuilderConfig,
};
use pipebuilder_common::{
    grpc::client::RepositoryClientBuilder, LocalBuildContextBuilder, Register, Result,
};
use tracing::info;

pub async fn bootstrap(
    node_id: String,
    external_address: String,
    config: BuilderConfig,
    lease_id: i64,
    register: Register,
) -> Result<BuilderService> {
    let repository_client_config = config.repository_client;
    let protocol = repository_client_config.protocol;
    let address = repository_client_config.address;
    let endpoint = format!("{}://{}", protocol, address);
    info!(
        endpoint = endpoint.as_str(),
        "connect repository service ..."
    );
    let repository_client = RepositoryClientBuilder::default()
        .protocol(protocol)
        .address(address.as_str())
        .connect()
        .await?;
    let workspace = config.workspace;
    let restore_directory = config.restore_directory;
    let log_directory = config.log_directory;
    let reset = config.reset.unwrap_or(true);
    let build_context = LocalBuildContextBuilder::default()
        .id(node_id)
        .address(external_address)
        .workspace(workspace)
        .restore_directory(restore_directory)
        .log_directory(log_directory)
        .build();
    let manager = BuildManager::builder()
        .lease_id(lease_id)
        .register(register)
        .repository_client(repository_client)
        .context(build_context)
        .build();
    manager.init(reset).await?;
    Ok(BuilderService::new(manager))
}
