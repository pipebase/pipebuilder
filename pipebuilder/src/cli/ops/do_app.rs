use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{AppMetadata, GetAppRequest, GetAppResponse, ListAppMetadataRequest},
    },
    Result,
};

pub(crate) async fn get_app(
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
    client.get_app(&request).await
}

pub(crate) async fn list_app_metadata(
    client: &ApiClient,
    namespace: String,
    id: Option<String>,
) -> Result<Vec<AppMetadata>> {
    let request = ListAppMetadataRequest { namespace, id };
    client.list_app_metadata(&request).await
}
