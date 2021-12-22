use super::print::Printer;
use pipebuilder_common::{
    api::{client::ApiClient, models},
    Result,
};

pub(crate) async fn push_catalogs(
    client: &ApiClient,
    namespace: String,
    id: String,
    buffer: Vec<u8>,
) -> Result<models::PostCatalogsResponse> {
    let request = models::PostCatalogsRequest {
        namespace,
        id,
        buffer,
    };
    client.push_catalogs(&request).await
}

pub(crate) async fn pull_catalogs(
    client: &ApiClient,
    namespace: String,
    id: String,
    version: u64,
) -> Result<models::GetCatalogsResponse> {
    let request = models::GetCatalogsRequest {
        namespace,
        id,
        version,
    };
    client.pull_catalogs(&request).await
}

pub(crate) async fn list_catalogs_snapshot(
    client: &ApiClient,
    namespace: String,
) -> Result<Vec<models::CatalogsSnapshot>> {
    let request = models::ListCatalogsSnapshotRequest { namespace };
    client.list_catalogs_snapshot(&request).await
}

pub(crate) async fn delete_catalogs(
    client: &ApiClient,
    namespace: String,
    id: String,
    version: u64,
) -> Result<()> {
    let request = models::DeleteCatalogsRequest {
        namespace,
        id,
        version,
    };
    client.delete_catalogs(&request).await
}

pub(crate) async fn delete_catalogs_snapshot(
    client: &ApiClient,
    namespace: String,
    id: String,
) -> Result<()> {
    let request = models::DeleteCatalogsSnapshotRequest { namespace, id };
    client.delete_catalogs_snapshot(&request).await
}

pub(crate) async fn delete_catalogs_all(
    client: &ApiClient,
    namespace: String,
    id: String,
) -> Result<()> {
    let mut printer = Printer::new();
    for manifest_metadata in
        list_catalogs_metadata(client, namespace.clone(), Some(id.clone())).await?
    {
        let id = manifest_metadata.id;
        let version = manifest_metadata.version;
        printer.status(
            "Deleting",
            format!(
                "catalogs (namespace = {}, id = {}, version = {})",
                namespace, id, version
            ),
        )?;
        delete_catalogs(client, namespace.clone(), id, version).await?;
    }
    // delete manifest snapshot
    printer.status(
        "Deleting",
        format!("catalogs snapshot (namespace = {}, id = {})", namespace, id),
    )?;
    delete_catalogs_snapshot(client, namespace.clone(), id.clone()).await
}

pub(crate) async fn list_catalogs_metadata(
    client: &ApiClient,
    namespace: String,
    id: Option<String>,
) -> Result<Vec<models::CatalogsMetadata>> {
    let request = models::ListCatalogsMetadataRequest { namespace, id };
    client.list_catalogs_metadata(&request).await
}

pub(crate) fn validate_catalogs(schema: &[u8]) -> Result<()> {
    ApiClient::validate_catalogs(schema)?;
    Ok(())
}
