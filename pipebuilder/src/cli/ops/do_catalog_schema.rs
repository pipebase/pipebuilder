use super::print::Printer;
use pipebuilder_common::{
    api::{client::ApiClient, models},
    Result,
};

pub(crate) async fn push_catalog_schema(
    client: &ApiClient,
    namespace: String,
    id: String,
    buffer: Vec<u8>,
) -> Result<models::PostCatalogSchemaResponse> {
    let request = models::PostCatalogSchemaRequest {
        namespace,
        id,
        buffer,
    };
    client.push_catalog_schema(&request).await
}

pub(crate) async fn pull_catalog_schema(
    client: &ApiClient,
    namespace: String,
    id: String,
    version: u64,
) -> Result<models::GetCatalogSchemaResponse> {
    let request = models::GetCatalogSchemaRequest {
        namespace,
        id,
        version,
    };
    client.pull_catalog_schema(&request).await
}

pub(crate) async fn list_catalog_schema_snapshot(
    client: &ApiClient,
    namespace: String,
) -> Result<Vec<models::CatalogSchemaSnapshot>> {
    let request = models::ListCatalogSchemaSnapshotRequest { namespace };
    client.list_catalog_schema_snapshot(&request).await
}

pub(crate) async fn delete_catalog_schema(
    client: &ApiClient,
    namespace: String,
    id: String,
    version: u64,
) -> Result<()> {
    let request = models::DeleteCatalogSchemaRequest {
        namespace,
        id,
        version,
    };
    client.delete_catalog_schema(&request).await
}

pub(crate) async fn delete_catalog_schema_snapshot(
    client: &ApiClient,
    namespace: String,
    id: String,
) -> Result<()> {
    let request = models::DeleteCatalogSchemaSnapshotRequest { namespace, id };
    client.delete_catalog_schema_snapshot(&request).await
}

pub(crate) async fn delete_catalog_schema_all(
    client: &ApiClient,
    namespace: String,
    id: String,
) -> Result<()> {
    let mut printer = Printer::new();
    for manifest_metadata in
        list_catalog_schema_metadata(client, namespace.clone(), Some(id.clone())).await?
    {
        let id = manifest_metadata.id;
        let version = manifest_metadata.version;
        printer.status(
            "Deleting",
            format!(
                "catalog schema (namespace = {}, id = {}, version = {})",
                namespace, id, version
            ),
        )?;
        delete_catalog_schema(client, namespace.clone(), id, version).await?;
    }
    // delete manifest snapshot
    printer.status(
        "Deleting",
        format!(
            "catalog schema snapshot (namespace = {}, id = {})",
            namespace, id
        ),
    )?;
    delete_catalog_schema_snapshot(client, namespace.clone(), id.clone()).await
}

pub(crate) async fn list_catalog_schema_metadata(
    client: &ApiClient,
    namespace: String,
    id: Option<String>,
) -> Result<Vec<models::CatalogSchemaMetadata>> {
    let request = models::ListCatalogSchemaMetadataRequest { namespace, id };
    client.list_catalog_schema_metadata(&request).await
}

pub(crate) fn validate_catalog_schema(schema: &[u8]) -> Result<()> {
    ApiClient::validate_catalog_schema(schema)?;
    Ok(())
}
