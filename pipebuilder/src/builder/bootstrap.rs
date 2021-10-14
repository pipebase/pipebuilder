use crate::{build::BuilderService, config::BuilderConfig};
use pipebuilder_common::{
    grpc::{build::builder_server::BuilderServer, repository::repository_client::RepositoryClient},
    LocalBuildContext, Register, Result,
};
use tonic::transport::Channel;

fn build_builder_service(
    lease_id: i64,
    register: Register,
    repository_client: RepositoryClient<Channel>,
    context: LocalBuildContext,
) -> BuilderService {
    BuilderService::new(lease_id, register, repository_client, context)
}

async fn build_repository_client(endpoint: String) -> Result<RepositoryClient<Channel>> {
    let repository_client = RepositoryClient::connect(endpoint).await?;
    Ok(repository_client)
}

pub async fn bootstrap(
    node_id: String,
    external_address: String,
    config: BuilderConfig,
    lease_id: i64,
    register: Register,
) -> Result<BuilderServer<BuilderService>> {
    let repository_endpoint = config.repository_endpoint;
    let repository_client = build_repository_client(repository_endpoint).await?;
    let workspace = config.workspace;
    let restore_directory = config.restore_directory;
    let log_directory = config.log_directory;
    let build_context = LocalBuildContext::new(
        node_id,
        external_address,
        workspace,
        restore_directory,
        log_directory,
    );
    let builder_svc = build_builder_service(lease_id, register, repository_client, build_context);
    Ok(BuilderServer::new(builder_svc))
}
