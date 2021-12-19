use pipebuilder_common::Register;

use crate::config::RepositoryConfig;
use crate::repository::{RepositoryService, RepositoryServiceBuilder};

pub fn bootstrap(config: RepositoryConfig, register: Register, lease_id: i64) -> RepositoryService {
    RepositoryServiceBuilder::default()
        .register(register)
        .lease_id(lease_id)
        .app_repository(config.app)
        .manifest_repository(config.manifest)
        .catalog_schema_repository(config.catalog_schema)
        .catalogs_repository(config.catalogs)
        .build()
}
