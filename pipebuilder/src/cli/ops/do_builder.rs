use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{BuildMetadataKey, ScanBuilderRequest},
    },
    Result,
};

pub(crate) async fn scan_builder(
    client: &ApiClient,
    builder_id: &str,
) -> Result<Vec<BuildMetadataKey>> {
    let request = ScanBuilderRequest {
        id: builder_id.to_owned(),
    };
    client.scan_builder(&request).await
}
