use crate::{build::BuilderService, config::BuilderConfig};
use pipebuilder_common::{
    grpc::{build::builder_server::BuilderServer, manifest::manifest_client::ManifestClient},
    Register, Result,
};
use tonic::transport::Channel;

fn build_builder_service(
    lease_id: i64,
    register: Register,
    manifest_client: ManifestClient<Channel>,
) -> BuilderService {
    BuilderService::new(lease_id, register, manifest_client)
}

async fn build_manifest_client(endpoint: String) -> Result<ManifestClient<Channel>> {
    let manifest_client = ManifestClient::connect(endpoint).await?;
    Ok(manifest_client)
}

pub async fn bootstrap(
    config: BuilderConfig,
    lease_id: i64,
    register: Register,
) -> Result<BuilderServer<BuilderService>> {
    let manifest_endpoint = config.manifest_endpoint;
    let manifest_client = build_manifest_client(manifest_endpoint).await?;
    let builder_svc = build_builder_service(lease_id, register, manifest_client);
    Ok(BuilderServer::new(builder_svc))
}
