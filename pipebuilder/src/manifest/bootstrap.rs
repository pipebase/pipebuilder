use pipebuilder_common::Register;

use crate::config::ManifestConfig;
use crate::manifest::ManifestService;
use pipebuilder_common::grpc::manifest::manifest_server::ManifestServer;

fn build_manifest_service(
    register: Register,
    lease_id: i64,
    repository: String,
) -> ManifestService {
    ManifestService::new(register, lease_id, repository)
}

pub fn bootstrap(
    config: ManifestConfig,
    register: Register,
    lease_id: i64,
) -> ManifestServer<ManifestService> {
    let repository = config.repository;
    let manifest_svc = build_manifest_service(register, lease_id, repository);
    ManifestServer::new(manifest_svc)
}
