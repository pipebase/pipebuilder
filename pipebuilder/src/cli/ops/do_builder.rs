use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{ScanBuilderRequest, VersionBuildKey},
    },
    Result,
};

pub(crate) async fn scan_builder(
    client: &ApiClient,
    builder_id: &str,
) -> Result<Vec<VersionBuildKey>> {
    let request = ScanBuilderRequest {
        id: builder_id.to_owned(),
    };
    client.scan_builder(&request).await
}
