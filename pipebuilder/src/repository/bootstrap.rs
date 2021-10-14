use pipebuilder_common::Register;

use crate::config::RepositoryConfig;
use crate::repository::RepositoryService;
use pipebuilder_common::grpc::repository::repository_server::RepositoryServer;

fn build_repository_service(
    register: Register,
    lease_id: i64,
    app_repository: String,
    manifest_repository: String,
) -> RepositoryService {
    RepositoryService::new(register, lease_id, app_repository, manifest_repository)
}

pub fn bootstrap(
    config: RepositoryConfig,
    register: Register,
    lease_id: i64,
) -> RepositoryServer<RepositoryService> {
    let app_repository = config.app;
    let manifest_repository = config.manifest;
    let manifest_svc =
        build_repository_service(register, lease_id, app_repository, manifest_repository);
    RepositoryServer::new(manifest_svc)
}
