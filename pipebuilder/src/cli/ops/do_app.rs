use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{GetAppRequest, GetAppResponse},
    },
    Result,
};

pub(crate) async fn get_app(
    client: &ApiClient,
    namespace: String,
    id: String,
    version: u64,
) -> Result<GetAppResponse> {
    let request = GetAppRequest {
        namespace,
        id,
        version,
    };
    client.get_app(&request).await
}
