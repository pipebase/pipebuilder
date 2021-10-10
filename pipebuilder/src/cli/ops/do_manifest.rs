use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{
            GetManifestRequest, GetManifestResponse, ListManifestSnapshotRequest, ManifestSnapshot,
            PutManifestRequest, PutManifestResponse,
        },
    },
    Result,
};
use std::fs;

pub(crate) async fn put_manifest(
    client: &ApiClient,
    namespace: String,
    id: String,
    buffer: Vec<u8>,
) -> Result<PutManifestResponse> {
    let request = PutManifestRequest {
        namespace,
        id,
        buffer,
    };
    client.put_manifest(&request).await
}

pub(crate) async fn get_manifest(
    client: &ApiClient,
    namespace: String,
    id: String,
    version: u64,
) -> Result<GetManifestResponse> {
    let request = GetManifestRequest {
        namespace,
        id,
        version,
    };
    client.get_manifest(&request).await
}

pub(crate) async fn list_manifest_snapshot(
    client: &ApiClient,
    namespace: String,
) -> Result<Vec<ManifestSnapshot>> {
    let request = ListManifestSnapshotRequest { namespace };
    client.list_manifest_snapshot(&request).await
}

pub(crate) fn validate_manifest(client: &ApiClient, path: &str) -> Result<()> {
    client.validate_manifest(path)?;
    Ok(())
}

pub(crate) fn read_file(path: &str) -> Result<Vec<u8>> {
    let buffer = fs::read(path)?;
    Ok(buffer)
}
