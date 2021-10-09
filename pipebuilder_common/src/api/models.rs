use super::constants::{
    DISPLAY_BUILD_STATUS_WIDTH, DISPLAY_ID_WIDTH, DISPLAY_MESSAGE_WIDTH, DISPLAY_TIMESTAMP_WIDTH,
    DISPLAY_VERSION_WIDTH,
};
use crate::{
    api::constants::DISPLAY_ADDRESS_WIDTH,
    grpc::{build, manifest},
    BuildStatus, Error,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

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
    // build version
    pub version: u64,
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

impl Display for ManifestSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{id:>id_width$}{latest_version:>version_width$}",
            id = self.id,
            latest_version = self.latest_version,
            id_width = DISPLAY_ID_WIDTH,
            version_width = DISPLAY_VERSION_WIDTH,
        )
    }
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

impl Display for BuildSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{id:>id_width$}{latest_version:>version_width$}",
            id = self.id,
            latest_version = self.latest_version,
            id_width = DISPLAY_ID_WIDTH,
            version_width = DISPLAY_VERSION_WIDTH,
        )
    }
}

// version build model for rest api
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

impl Display for VersionBuild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self.message.as_ref() {
            Some(message) => message.as_str(),
            None => "",
        };
        writeln!(f,
                "{id:>id_width$}{version:>version_width$}{status:>status_width$}{timestamp:>timestamp_width$}{builder_id:>id_width$}{builder_address:>address_width$}{message:>message_width$}",
                id = self.id,
                version = self.version,
                status = self.status,
                timestamp = self.timestamp,
                builder_id = self.builder_id,
                builder_address = self.builder_address,
                message = message,
                id_width = DISPLAY_ID_WIDTH,
                version_width = DISPLAY_VERSION_WIDTH,
                status_width = DISPLAY_BUILD_STATUS_WIDTH,
                timestamp_width = DISPLAY_TIMESTAMP_WIDTH,
                address_width = DISPLAY_ADDRESS_WIDTH,
                message_width = DISPLAY_MESSAGE_WIDTH,
                )
    }
}

#[derive(Serialize, Deserialize)]
pub struct ListBuildRequest {
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
