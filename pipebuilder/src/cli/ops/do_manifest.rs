use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{
            GetManifestRequest, GetManifestResponse, PutManifestRequest, PutManifestResponse,
        },
    },
    Result,
};

pub(crate) async fn put_manifest(
    client: ApiClient,
    namespace: String,
    id: String,
    buffer: Vec<u8>,
) -> Result<PutManifestResponse> {
    let request = PutManifestRequest {
        namespace,
        id,
        buffer,
    };
    client.put_manifest(&request).await
}

pub(crate) async fn get_manifest(
    client: ApiClient,
    namespace: String,
    id: String,
    version: u64,
) -> Result<GetManifestResponse> {
    let request = GetManifestRequest {
        namespace,
        id,
        version,
    };
    client.get_manifest(&request).await
}
