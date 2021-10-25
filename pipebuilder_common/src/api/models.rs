use crate::{
    api::constants::{
        DISPLAY_ADDRESS_WIDTH, DISPLAY_BUILD_STATUS_WIDTH, DISPLAY_COUNT_WIDTH, DISPLAY_ID_WIDTH,
        DISPLAY_MESSAGE_WIDTH, DISPLAY_NAMESPACE_WIDTH, DISPLAY_NODE_ROLE_WIDTH,
        DISPLAY_NODE_STATUS_WIDTH, DISPLAY_SIZE_WIDTH, DISPLAY_TIMESTAMP_WIDTH,
        DISPLAY_VERSION_WIDTH,
    },
    grpc::{build, node, repository},
    BuildStatus, Error, NodeRole, NodeState as InternalNodeState, NodeStatus,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub trait PrintHeader {
    fn print_header();
}

#[derive(Serialize, Deserialize)]
pub struct BuildRequest {
    pub namespace: String,
    // project id
    pub id: String,
    pub manifest_version: u64,
    pub target_platform: String,
}

#[derive(Serialize, Deserialize)]
pub struct BuildResponse {
    pub build_version: u64,
}

impl Display for BuildResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "build version: {}", self.build_version)
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetBuildRequest {
    // namespace
    pub namespace: String,
    // project id
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

impl Display for PutManifestResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{id:<id_width$}{version:<version_width$}",
            id = self.id,
            version = self.version,
            id_width = DISPLAY_ID_WIDTH,
            version_width = DISPLAY_VERSION_WIDTH,
        )
    }
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
            "{id:<id_width$}{latest_version:<version_width$}",
            id = self.id,
            latest_version = self.latest_version,
            id_width = DISPLAY_ID_WIDTH,
            version_width = DISPLAY_VERSION_WIDTH,
        )
    }
}

impl PrintHeader for ManifestSnapshot {
    fn print_header() {
        println!(
            "{col0:<col0_width$}{col1:<col1_width$}",
            col0 = "Id",
            col1 = "Latest Version",
            col0_width = DISPLAY_ID_WIDTH,
            col1_width = DISPLAY_VERSION_WIDTH,
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
            "{id:<id_width$}{latest_version:<version_width$}",
            id = self.id,
            latest_version = self.latest_version,
            id_width = DISPLAY_ID_WIDTH,
            version_width = DISPLAY_VERSION_WIDTH,
        )
    }
}

impl PrintHeader for BuildSnapshot {
    fn print_header() {
        println!(
            "{col0:<col0_width$}{col1:<col1_width$}",
            col0 = "Id",
            col1 = "Latest Version",
            col0_width = DISPLAY_ID_WIDTH,
            col1_width = DISPLAY_VERSION_WIDTH,
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
        let status = self.status.to_string();
        let timestamp = self.timestamp.to_string();
        writeln!(f,
                "{id:<id_width$}{version:<version_width$}{status:<status_width$}{builder_id:<id_width$}{builder_address:<address_width$}{timestamp:<timestamp_width$}{message:<message_width$}",
                id = self.id,
                version = self.version,
                status = status,
                timestamp = timestamp,
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

impl PrintHeader for VersionBuild {
    fn print_header() {
        println!(
            "{col0:<col0_width$}{col1:<col1_width$}{col2:<col2_width$}{col3:<col3_width$}{col4:<col4_width$}{col5:<col5_width$}{col6:<col6_width$}",
            col0 = "Id",
            col1 = "Version",
            col2 = "Status",
            col3 = "Builder Id",
            col4 = "Builder Address",
            col5 = "Timestamp",
            col6 = "Message",
            col0_width = DISPLAY_ID_WIDTH,
            col1_width = DISPLAY_VERSION_WIDTH,
            col2_width = DISPLAY_BUILD_STATUS_WIDTH,
            col3_width = DISPLAY_ID_WIDTH,
            col4_width = DISPLAY_ADDRESS_WIDTH,
            col5_width = DISPLAY_TIMESTAMP_WIDTH,
            col6_width = DISPLAY_MESSAGE_WIDTH,
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct ListBuildRequest {
    pub namespace: String,
    // project id
    pub id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CancelBuildRequest {
    pub namespace: String,
    pub id: String,
    pub version: u64,
}

#[derive(Serialize, Deserialize)]
pub struct CancelBuildResponse {}

#[derive(Serialize, Deserialize)]
pub struct GetAppRequest {
    pub namespace: String,
    pub id: String,
    pub build_version: u64,
}

#[derive(Serialize, Deserialize)]
pub struct GetAppResponse {
    pub buffer: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetBuildLogRequest {
    pub namespace: String,
    pub id: String,
    pub version: u64,
}

#[derive(Serialize, Deserialize)]
pub struct GetBuildLogResponse {
    pub buffer: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct ListNodeStateRequest {
    pub role: Option<NodeRole>,
}

#[derive(Serialize, Deserialize)]
pub struct NodeState {
    // node id
    pub id: String,
    // node role
    pub role: NodeRole,
    // status
    pub status: NodeStatus,
    // timestamp
    pub timestamp: DateTime<Utc>,
}

impl Display for NodeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let role = self.role.to_string();
        let status = self.status.to_string();
        let timestamp = self.timestamp.to_string();
        writeln!(f,
                "{id:<id_width$}{role:<role_width$}{status:<status_width$}{timestamp:<timestamp_width$}",
                id = self.id,
                role = role,
                status = status,
                timestamp = timestamp,
                id_width = DISPLAY_ID_WIDTH,
                role_width = DISPLAY_NODE_ROLE_WIDTH,
                status_width = DISPLAY_NODE_STATUS_WIDTH,
                timestamp_width = DISPLAY_TIMESTAMP_WIDTH,
                )
    }
}

impl PrintHeader for NodeState {
    fn print_header() {
        println!(
            "{col0:<col0_width$}{col1:<col1_width$}{col2:<col2_width$}{col3:<col3_width$}",
            col0 = "Id",
            col1 = "Role",
            col2 = "Status",
            col3 = "Timestamp",
            col0_width = DISPLAY_ID_WIDTH,
            col1_width = DISPLAY_NODE_ROLE_WIDTH,
            col2_width = DISPLAY_NODE_STATUS_WIDTH,
            col3_width = DISPLAY_TIMESTAMP_WIDTH,
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct ScanBuilderRequest {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct VersionBuildKey {
    pub namespace: String,
    pub id: String,
    pub version: u64,
}

impl Display for VersionBuildKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{namespace:<namespace_width$}{id:<id_width$}{version:<version_width$}",
            namespace = self.namespace,
            id = self.id,
            version = self.version,
            namespace_width = DISPLAY_NAMESPACE_WIDTH,
            id_width = DISPLAY_ID_WIDTH,
            version_width = DISPLAY_VERSION_WIDTH,
        )
    }
}

impl PrintHeader for VersionBuildKey {
    fn print_header() {
        println!(
            "{col0:<col0_width$}{col1:<col1_width$}{col2:<col2_width$}",
            col0 = "Namespace",
            col1 = "Id",
            col2 = "Version",
            col0_width = DISPLAY_NAMESPACE_WIDTH,
            col1_width = DISPLAY_ID_WIDTH,
            col2_width = DISPLAY_VERSION_WIDTH,
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct ActivateNodeRequest {
    pub role: NodeRole,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ActivateNodeResponse {}

#[derive(Serialize, Deserialize)]
pub struct DeactivateNodeRequest {
    pub role: NodeRole,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeactivateNodeResponse {}

#[derive(Serialize, Deserialize)]
pub struct ListAppMetadataRequest {
    pub namespace: String,
    pub id: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AppMetadata {
    // project id
    pub id: String,
    // build version
    pub version: u64,
    pub pulls: u64,
    pub size: usize,
    pub created: DateTime<Utc>,
}

impl Display for AppMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{id:<id_width$}{version:<version_width$}{pulls:<pulls_width$}{size:<size_width$}{created:<created_width$}",
            id = self.id,
            version = self.version,
            pulls = self.pulls,
            size = self.size,
            created = self.created,
            id_width = DISPLAY_ID_WIDTH,
            version_width = DISPLAY_VERSION_WIDTH,
            pulls_width = DISPLAY_COUNT_WIDTH,
            size_width = DISPLAY_SIZE_WIDTH,
            created_width = DISPLAY_TIMESTAMP_WIDTH,
        )
    }
}

impl PrintHeader for AppMetadata {
    fn print_header() {
        println!(
            "{col0:<col0_width$}{col1:<col1_width$}{col2:<col2_width$}{col3:<col3_width$}{col4:<col4_width$}",
            col0 = "Id",
            col1 = "Version",
            col2 = "Pulls",
            col3 = "Size",
            col4 = "Created",
            col0_width = DISPLAY_ID_WIDTH,
            col1_width = DISPLAY_VERSION_WIDTH,
            col2_width = DISPLAY_COUNT_WIDTH,
            col3_width = DISPLAY_SIZE_WIDTH,
            col4_width = DISPLAY_TIMESTAMP_WIDTH,
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct ListManifestMetadataRequest {
    pub namespace: String,
    pub id: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ManifestMetadata {
    // project id
    pub id: String,
    // manifest version
    pub version: u64,
    pub pulls: u64,
    pub size: usize,
    pub created: DateTime<Utc>,
}

impl Display for ManifestMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{id:<id_width$}{version:<version_width$}{pulls:<pulls_width$}{size:<size_width$}{created:<created_width$}",
            id = self.id,
            version = self.version,
            pulls = self.pulls,
            size = self.size,
            created = self.created,
            id_width = DISPLAY_ID_WIDTH,
            version_width = DISPLAY_VERSION_WIDTH,
            pulls_width = DISPLAY_COUNT_WIDTH,
            size_width = DISPLAY_SIZE_WIDTH,
            created_width = DISPLAY_TIMESTAMP_WIDTH,
        )
    }
}

impl PrintHeader for ManifestMetadata {
    fn print_header() {
        println!(
            "{col0:<col0_width$}{col1:<col1_width$}{col2:<col2_width$}{col3:<col3_width$}{col4:<col4_width$}",
            col0 = "Id",
            col1 = "Version",
            col2 = "Pulls",
            col3 = "Size",
            col4 = "Created",
            col0_width = DISPLAY_ID_WIDTH,
            col1_width = DISPLAY_VERSION_WIDTH,
            col2_width = DISPLAY_COUNT_WIDTH,
            col3_width = DISPLAY_SIZE_WIDTH,
            col4_width = DISPLAY_TIMESTAMP_WIDTH,
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateNamespaceRequest {
    // namespace id
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListNamespaceRequest {}

#[derive(Serialize, Deserialize)]
pub struct Namespace {
    pub id: String,
    pub created: DateTime<Utc>,
}

impl Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{id:<id_width$}{created:<created_width$}",
            id = self.id,
            created = self.created,
            id_width = DISPLAY_ID_WIDTH,
            created_width = DISPLAY_TIMESTAMP_WIDTH,
        )
    }
}

impl PrintHeader for Namespace {
    fn print_header() {
        println!(
            "{col0:<col0_width$}{col1:<col1_width$}",
            col0 = "Id",
            col1 = "Created",
            col0_width = DISPLAY_ID_WIDTH,
            col1_width = DISPLAY_TIMESTAMP_WIDTH,
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub namespace: String,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListProjectRequest {
    pub namespace: String,
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    // project id
    pub id: String,
    pub created: DateTime<Utc>,
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{id:<id_width$}{created:<created_width$}",
            id = self.id,
            created = self.created,
            id_width = DISPLAY_ID_WIDTH,
            created_width = DISPLAY_TIMESTAMP_WIDTH,
        )
    }
}

impl PrintHeader for Project {
    fn print_header() {
        println!(
            "{col0:<col0_width$}{col1:<col1_width$}",
            col0 = "Id",
            col1 = "Created",
            col0_width = DISPLAY_ID_WIDTH,
            col1_width = DISPLAY_TIMESTAMP_WIDTH,
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
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

impl From<CancelBuildRequest> for build::CancelRequest {
    fn from(origin: CancelBuildRequest) -> Self {
        let namespace = origin.namespace;
        let id = origin.id;
        let version = origin.version;
        build::CancelRequest {
            namespace,
            id,
            build_version: version,
        }
    }
}

impl From<build::CancelResponse> for CancelBuildResponse {
    fn from(_origin: build::CancelResponse) -> Self {
        CancelBuildResponse {}
    }
}

impl From<PutManifestRequest> for repository::PutManifestRequest {
    fn from(origin: PutManifestRequest) -> Self {
        let namespace = origin.namespace;
        let id = origin.id;
        let buffer = origin.buffer;
        repository::PutManifestRequest {
            namespace,
            id,
            buffer,
        }
    }
}

impl From<repository::PutManifestResponse> for PutManifestResponse {
    fn from(origin: repository::PutManifestResponse) -> Self {
        let id = origin.id;
        let version = origin.version;
        PutManifestResponse { id, version }
    }
}

impl From<GetManifestRequest> for repository::GetManifestRequest {
    fn from(origin: GetManifestRequest) -> Self {
        let namespace = origin.namespace;
        let id = origin.id;
        let version = origin.version;
        repository::GetManifestRequest {
            namespace,
            id,
            version,
        }
    }
}

impl From<repository::GetManifestResponse> for GetManifestResponse {
    fn from(origin: repository::GetManifestResponse) -> Self {
        let buffer = origin.buffer;
        GetManifestResponse { buffer }
    }
}

impl From<GetAppRequest> for repository::GetAppRequest {
    fn from(origin: GetAppRequest) -> Self {
        let namespace = origin.namespace;
        let id = origin.id;
        let version = origin.build_version;
        repository::GetAppRequest {
            namespace,
            id,
            version,
        }
    }
}

impl From<repository::GetAppResponse> for GetAppResponse {
    fn from(origin: repository::GetAppResponse) -> Self {
        let buffer = origin.buffer;
        GetAppResponse { buffer }
    }
}

impl From<GetBuildLogRequest> for build::GetLogRequest {
    fn from(origin: GetBuildLogRequest) -> Self {
        let namespace = origin.namespace;
        let id = origin.id;
        let build_version = origin.version;
        build::GetLogRequest {
            namespace,
            id,
            build_version,
        }
    }
}

impl From<build::GetLogResponse> for GetBuildLogResponse {
    fn from(origin: build::GetLogResponse) -> Self {
        let buffer = origin.buffer;
        GetBuildLogResponse { buffer }
    }
}

impl From<InternalNodeState> for NodeState {
    fn from(origin: InternalNodeState) -> Self {
        let id = origin.id;
        let role = origin.role;
        let status = origin.status;
        let timestamp = origin.timestamp;
        NodeState {
            id,
            role,
            status,
            timestamp,
        }
    }
}

impl From<build::VersionBuildKey> for VersionBuildKey {
    fn from(origin: build::VersionBuildKey) -> Self {
        let namespace = origin.namespace;
        let id = origin.id;
        let version = origin.version;
        VersionBuildKey {
            namespace,
            id,
            version,
        }
    }
}

impl From<ScanBuilderRequest> for build::ScanRequest {
    fn from(_: ScanBuilderRequest) -> Self {
        build::ScanRequest {}
    }
}

impl From<node::ActivateResponse> for ActivateNodeResponse {
    fn from(_: node::ActivateResponse) -> Self {
        ActivateNodeResponse {}
    }
}

impl From<node::DeactivateResponse> for DeactivateNodeResponse {
    fn from(_: node::DeactivateResponse) -> Self {
        DeactivateNodeResponse {}
    }
}

impl From<Error> for Failure {
    fn from(error: Error) -> Self {
        Failure::new(format!("{}", error))
    }
}
