use super::print::Printer;
use pipebuilder_common::{
    api::{client::ApiClient, models},
    Result,
};

pub(crate) async fn push_manifest(
    client: &ApiClient,
    namespace: String,
    id: String,
    buffer: Vec<u8>,
) -> Result<models::PostManifestResponse> {
    let request = models::PostManifestRequest {
        namespace,
        id,
        buffer,
    };
    client.push_manifest(&request).await
}

pub(crate) async fn pull_manifest(
    client: &ApiClient,
    namespace: String,
    id: String,
    version: u64,
) -> Result<models::GetManifestResponse> {
    let request = models::GetManifestRequest {
        namespace,
        id,
        version,
    };
    client.pull_manifest(&request).await
}

pub(crate) async fn list_manifest_snapshot(
    client: &ApiClient,
    namespace: String,
) -> Result<Vec<models::ManifestSnapshot>> {
    let request = models::ListManifestSnapshotRequest { namespace };
    client.list_manifest_snapshot(&request).await
}

pub(crate) async fn delete_manifest(
    client: &ApiClient,
    namespace: String,
    id: String,
    version: u64,
) -> Result<()> {
    let request = models::DeleteManifestRequest {
        namespace,
        id,
        version,
    };
    client.delete_manfiest(&request).await
}

pub(crate) async fn delete_manifest_snapshot(
    client: &ApiClient,
    namespace: String,
    id: String,
) -> Result<()> {
    let request = models::DeleteManifestSnapshotRequest { namespace, id };
    client.delete_manifest_snapshot(&request).await
}

pub(crate) async fn delete_manifest_all(
    client: &ApiClient,
    namespace: String,
    id: String,
) -> Result<()> {
    let mut printer = Printer::new();
    for manifest_metadata in
        list_manifest_metadata(client, namespace.clone(), Some(id.clone())).await?
    {
        let id = manifest_metadata.id;
        let version = manifest_metadata.version;
        printer.status(
            "Deleting",
            format!(
                "manifest (namespace = {}, id = {}, version = {})",
                namespace, id, version
            ),
        )?;
        delete_manifest(client, namespace.clone(), id, version).await?;
    }
    // delete manifest snapshot
    printer.status(
        "Deleting",
        format!("manifest snapshot (namespace = {}, id = {})", namespace, id),
    )?;
    delete_manifest_snapshot(client, namespace.clone(), id.clone()).await
}

pub(crate) async fn list_manifest_metadata(
    client: &ApiClient,
    namespace: String,
    id: Option<String>,
) -> Result<Vec<models::ManifestMetadata>> {
    let request = models::ListManifestMetadataRequest { namespace, id };
    client.list_manifest_metadata(&request).await
}

pub(crate) fn validate_manifest(manifest: &[u8]) -> Result<()> {
    ApiClient::validate_manifest(manifest)?;
    Ok(())
}
