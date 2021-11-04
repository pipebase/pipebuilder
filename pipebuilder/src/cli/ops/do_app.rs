use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{
            AppMetadata, DeleteAppRequest, GetAppRequest, GetAppResponse, ListAppMetadataRequest,
        },
    },
    Result,
};

pub(crate) async fn pull_app(
    client: &ApiClient,
    namespace: String,
    id: String,
    build_version: u64,
) -> Result<GetAppResponse> {
    let request = GetAppRequest {
        namespace,
        id,
        build_version,
    };
    client.pull_app(&request).await
}

pub(crate) async fn list_app_metadata(
    client: &ApiClient,
    namespace: String,
    id: Option<String>,
) -> Result<Vec<AppMetadata>> {
    let request = ListAppMetadataRequest { namespace, id };
    client.list_app_metadata(&request).await
}

pub(crate) async fn delete_app(
    client: &ApiClient,
    namespace: String,
    id: String,
    build_version: u64,
) -> Result<()> {
    let request = DeleteAppRequest {
        namespace,
        id,
        version: build_version,
    };
    client.delete_app(&request).await
}

pub(crate) async fn delete_app_all(
    client: &ApiClient,
    namespace: String,
    id: String,
) -> Result<()> {
    for app_metadata in list_app_metadata(client, namespace.clone(), Some(id.clone())).await? {
        let id = app_metadata.id;
        let version = app_metadata.version;
        delete_app(client, namespace.clone(), id.clone(), version).await?;
    }
    Ok(())
}
