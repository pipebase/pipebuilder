use pipebuilder_common::Register;

use crate::config::RepositoryConfig;
use crate::repository::RepositoryService;

fn build_repository_service(
    register: Register,
    lease_id: i64,
    app_repository: String,
    manifest_repository: String,
) -> RepositoryService {
    RepositoryService::new(register, lease_id, app_repository, manifest_repository)
}

pub fn bootstrap(config: RepositoryConfig, register: Register, lease_id: i64) -> RepositoryService {
    let app_repository = config.app;
    let manifest_repository = config.manifest;
    build_repository_service(register, lease_id, app_repository, manifest_repository)
}
