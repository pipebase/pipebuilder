use super::print::Printer;
use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{
            BuildMetadata, BuildRequest, BuildResponse, BuildSnapshot, CancelBuildRequest,
            CancelBuildResponse, DeleteBuildRequest, DeleteBuildSnapshotRequest,
            GetBuildLogRequest, GetBuildLogResponse, GetBuildRequest, ListBuildRequest,
            ListBuildSnapshotRequest,
        },
    },
    Result,
};

pub(crate) async fn build(
    client: &ApiClient,
    namespace: String,
    id: String,
    manifest_version: u64,
    target_platform: Option<String>,
) -> Result<BuildResponse> {
    let request = BuildRequest {
        namespace,
        id,
        manifest_version,
        target_platform,
    };
    client.build(&request).await
}

pub(crate) async fn get_build_metadata(
    client: &ApiClient,
    namespace: String,
    id: String,
    version: u64,
) -> Result<BuildMetadata> {
    let request = GetBuildRequest {
        namespace,
        id,
        version,
    };
    client.get_build_metadata(&request).await
}

pub(crate) async fn list_build_metadata(
    client: &ApiClient,
    namespace: String,
    id: Option<String>,
) -> Result<Vec<BuildMetadata>> {
    let request = ListBuildRequest { namespace, id };
    client.list_build_metadata(&request).await
}

pub(crate) async fn list_build_snapshot(
    client: &ApiClient,
    namespace: String,
) -> Result<Vec<BuildSnapshot>> {
    let request = ListBuildSnapshotRequest { namespace };
    client.list_build_snapshot(&request).await
}

pub(crate) async fn cancel_build(
    client: &ApiClient,
    namespace: String,
    id: String,
    version: u64,
) -> Result<CancelBuildResponse> {
    let request = CancelBuildRequest {
        namespace,
        id,
        version,
    };
    client.cancel_build(&request).await
}

pub(crate) async fn delete_build(
    client: &ApiClient,
    namespace: String,
    id: String,
    version: u64,
) -> Result<()> {
    let request = DeleteBuildRequest {
        namespace,
        id,
        version,
    };
    client.delete_build(&request).await
}

pub(crate) async fn delete_build_snapshot(
    client: &ApiClient,
    namespace: String,
    id: String,
) -> Result<()> {
    let request = DeleteBuildSnapshotRequest { namespace, id };
    client.delete_build_snapshot(&request).await
}

pub(crate) async fn pull_build_log(
    client: &ApiClient,
    namespace: String,
    id: String,
    version: u64,
) -> Result<GetBuildLogResponse> {
    let request = GetBuildLogRequest {
        namespace,
        id,
        version,
    };
    client.pull_build_log(&request).await
}

pub(crate) async fn delete_build_all(
    client: &ApiClient,
    namespace: String,
    id: String,
) -> Result<()> {
    let mut printer = Printer::new();
    for build_metadata in list_build_metadata(client, namespace.clone(), Some(id.clone())).await? {
        let id = build_metadata.id;
        let version = build_metadata.version;
        printer.status(
            "Deleting",
            format!("build '{}/{}/{}'", namespace, id, version),
        )?;
        delete_build(client, namespace.clone(), id, version).await?;
    }
    // delete build snapshot
    printer.status("Deleting", format!("build snapshot '{}/{}'", namespace, id))?;
    delete_build_snapshot(client, namespace.clone(), id.clone()).await?;
    Ok(())
}
