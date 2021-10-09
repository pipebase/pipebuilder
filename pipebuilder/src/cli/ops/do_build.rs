use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{BuildRequest, BuildResponse, GetBuildRequest, ListBuildRequest, VersionBuild},
    },
    Result,
};

pub(crate) async fn build(
    client: ApiClient,
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
    client: ApiClient,
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
    client: ApiClient,
    namespace: String,
    id: String,
) -> Result<Vec<VersionBuild>> {
    let request = ListBuildRequest { namespace, id };
    client.list_build(&request).await
}
