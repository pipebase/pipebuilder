use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{
            BuildRequest, BuildResponse, BuildSnapshot, CancelBuildRequest, CancelBuildResponse,
            GetBuildLogRequest, GetBuildLogResponse, GetBuildRequest, ListBuildRequest,
            ListBuildSnapshotRequest, VersionBuild,
        },
    },
    Result,
};

pub(crate) async fn build(
    client: &ApiClient,
    namespace: String,
    id: String,
    manifest_version: u64,
    target_platform: String,
) -> Result<BuildResponse> {
    let request = BuildRequest {
        namespace,
        id,
        manifest_version,
        target_platform,
    };
    client.build(&request).await
}

pub(crate) async fn get_build(
    client: &ApiClient,
    namespace: String,
    id: String,
    version: u64,
) -> Result<VersionBuild> {
    let request = GetBuildRequest {
        namespace,
        id,
        version,
    };
    client.get_build(&request).await
}

pub(crate) async fn list_build(
    client: &ApiClient,
    namespace: String,
    id: Option<String>,
) -> Result<Vec<VersionBuild>> {
    let request = ListBuildRequest { namespace, id };
    client.list_build(&request).await
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

pub(crate) async fn get_build_log(
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
    client.get_build_log(&request).await
}
