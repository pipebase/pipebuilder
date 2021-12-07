use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{BuildMetadataKey, ScanBuildRequest},
    },
    Result,
};

pub(crate) async fn scan_build(
    client: &ApiClient,
    builder_id: &str,
) -> Result<Vec<BuildMetadataKey>> {
    let request = ScanBuildRequest {
        builder_id: builder_id.to_owned(),
    };
    client.scan_build(&request).await
}
