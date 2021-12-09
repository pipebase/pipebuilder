use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{BuildCacheMetadata, BuildMetadataKey, ScanBuildCacheRequest, ScanBuildRequest},
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

pub(crate) async fn scan_build_cache(
    client: &ApiClient,
    builder_id: &str,
) -> Result<Vec<BuildCacheMetadata>> {
    let request = ScanBuildCacheRequest {
        builder_id: builder_id.to_owned(),
    };
    client.scan_build_cache(&request).await
}
