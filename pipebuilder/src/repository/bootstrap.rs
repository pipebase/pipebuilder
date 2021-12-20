use pipebuilder_common::{Register, Result};

use crate::config::RepositoryConfig;
use crate::repository::RepositoryService;

pub async fn bootstrap(
    config: RepositoryConfig,
    register: Register,
    lease_id: i64,
) -> Result<RepositoryService> {
    let svc = RepositoryService::builder()
        .register(register)
        .lease_id(lease_id)
        .app_directory(config.app)
        .manifest_directory(config.manifest)
        .catalog_schema_directory(config.catalog_schema)
        .catalogs_directory(config.catalogs)
        .build();
    let reset = config.reset.unwrap_or(false);
    svc.init(reset).await?;
    Ok(svc)
}
