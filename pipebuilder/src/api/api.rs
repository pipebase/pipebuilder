mod filters {}

mod handlers {}

mod models {

    use pipebuilder_common::BuildStatus;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct BuildRequest {
        pub namespace: String,
        pub manifest_id: String,
        pub manifest_version: u64,
        pub target_platform: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct BuildResponse {
        pub build_version: u64,
    }

    #[derive(Serialize, Deserialize)]
    pub struct GetBuildRequest {
        pub namespace: String,
        pub manifest_id: String,
        pub build_version: u64,
    }

    #[derive(Serialize, Deserialize)]
    pub struct GetBuildResponse {
        pub status: BuildStatus,
    }

    #[derive(Serialize, Deserialize)]
    pub struct PutManifestRequest {
        pub namespace: String,
        pub id: Option<String>,
        pub buffer: Vec<u8>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct PutManifestResponse {
        pub id: String,
        pub version: u64,
    }
}
