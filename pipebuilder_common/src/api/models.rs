use crate::{
    grpc::{build, manifest},
    BuildStatus, Error,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BuildRequest {
    pub namespace: String,
    pub id: String,
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
    pub id: String,
    pub build_version: u64,
}

#[derive(Serialize, Deserialize)]
pub struct GetBuildResponse {
    pub status: BuildStatus,
}

#[derive(Serialize, Deserialize)]
pub struct PutManifestRequest {
    pub namespace: String,
    pub id: String,
    pub buffer: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct PutManifestResponse {
    pub id: String,
    pub version: u64,
}

#[derive(Serialize, Deserialize)]
pub struct GetManifestRequest {
    pub namespace: String,
    pub id: String,
    pub version: u64,
}

#[derive(Serialize, Deserialize)]
pub struct GetManifestResponse {
    pub buffer: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct ListManifestSnapshotRequest {
    pub namespace: String,
}

#[derive(Serialize, Deserialize)]
pub struct ManifestSnapshot {
    pub id: String,
    pub latest_version: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ListBuildSnapshotRequest {
    pub namespace: String,
}

#[derive(Serialize, Deserialize)]
pub struct BuildSnapshot {
    pub id: String,
    pub latest_version: u64,
}

#[derive(Serialize, Deserialize)]
pub struct GetVersionBuildRequest {
    pub namespace: String,
    pub id: String,
    pub version: u64,
}

#[derive(Serialize, Deserialize)]
pub struct VersionBuild {
    // id
    pub id: String,
    // version
    pub version: u64,
    // build status
    pub status: BuildStatus,
    // timestamp
    pub timestamp: DateTime<Utc>,
    // builder id
    pub builder_id: String,
    // builder address
    pub builder_address: String,
    // message
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ListVersionBuildRequest {
    pub namespace: String,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct CancelBuildRequest {
    pub namespace: String,
    pub id: String,
    pub version: u64,
}

#[derive(Serialize, Deserialize)]
pub struct CancelBuildResponse {}

#[derive(Serialize, Deserialize)]
pub struct Failure {
    pub error: String,
}

impl Failure {
    pub fn new(error: String) -> Self {
        Failure { error }
    }
}

impl From<BuildRequest> for build::BuildRequest {
    fn from(origin: BuildRequest) -> Self {
        let namespace = origin.namespace;
        let id = origin.id;
        let manifest_version = origin.manifest_version;
        let target_platform = origin.target_platform;
        build::BuildRequest {
            namespace,
            id,
            manifest_version,
            target_platform,
        }
    }
}

impl From<build::BuildResponse> for BuildResponse {
    fn from(origin: build::BuildResponse) -> Self {
        let build_version = origin.version;
        BuildResponse { build_version }
    }
}

impl From<build::CancelResponse> for CancelBuildResponse {
    fn from(_origin: build::CancelResponse) -> Self {
        CancelBuildResponse {}
    }
}

impl From<PutManifestRequest> for manifest::PutManifestRequest {
    fn from(origin: PutManifestRequest) -> Self {
        let namespace = origin.namespace;
        let id = origin.id;
        let buffer = origin.buffer;
        manifest::PutManifestRequest {
            namespace,
            id,
            buffer,
        }
    }
}

impl From<manifest::PutManifestResponse> for PutManifestResponse {
    fn from(origin: manifest::PutManifestResponse) -> Self {
        let id = origin.id;
        let version = origin.version;
        PutManifestResponse { id, version }
    }
}

impl From<GetManifestRequest> for manifest::GetManifestRequest {
    fn from(origin: GetManifestRequest) -> Self {
        let namespace = origin.namespace;
        let id = origin.id;
        let version = origin.version;
        manifest::GetManifestRequest {
            namespace,
            id,
            version,
        }
    }
}

impl From<manifest::GetManifestResponse> for GetManifestResponse {
    fn from(origin: manifest::GetManifestResponse) -> Self {
        let buffer = origin.buffer;
        GetManifestResponse { buffer }
    }
}

impl From<Error> for Failure {
    fn from(error: Error) -> Self {
        Failure::new(format!("{:#?}", error))
    }
}
