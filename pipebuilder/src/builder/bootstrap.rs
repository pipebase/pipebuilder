use crate::{build::BuilderService, config::BuilderConfig};
use pipebuilder_common::{
    grpc::{build::builder_server::BuilderServer, manifest::manifest_client::ManifestClient},
    LocalBuildContext, Register, Result,
};
use tonic::transport::Channel;

fn build_builder_service(
    lease_id: i64,
    register: Register,
    manifest_client: ManifestClient<Channel>,
    context: LocalBuildContext,
) -> BuilderService {
    BuilderService::new(lease_id, register, manifest_client, context)
}

async fn build_manifest_client(endpoint: String) -> Result<ManifestClient<Channel>> {
    let manifest_client = ManifestClient::connect(endpoint).await?;
    Ok(manifest_client)
}

pub async fn bootstrap(
    node_id: String,
    external_address: String,
    config: BuilderConfig,
    lease_id: i64,
    register: Register,
) -> Result<BuilderServer<BuilderService>> {
    let manifest_endpoint = config.manifest_endpoint;
    let manifest_client = build_manifest_client(manifest_endpoint).await?;
    let workspace = config.workspace;
    let target_directory = config.target_directory;
    let build_log_directory = config.build_log_directory;
    let build_context = LocalBuildContext::new(
        node_id,
        external_address,
        workspace,
        target_directory,
        build_log_directory,
    );
    let builder_svc = build_builder_service(lease_id, register, manifest_client, build_context);
    Ok(BuilderServer::new(builder_svc))
}
