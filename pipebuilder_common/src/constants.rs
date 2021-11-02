use crate::Period;

pub const ENV_PIPEBUILDER_CONFIG_FILE: &str = "PIPEBUILDER_CONFIG_FILE";
pub const ENV_PIPEBUILDER_NODE_ID: &str = "PIPEBUILDER_NODE_ID";
pub const ENV_PIPEBUILDER_EXTERNAL_ADDR: &str = "PIPEBUILDER_EXTERNAL_ADDR";
pub const DEFAULT_NODE_HEARTBEAT_PERIOD: Period = Period::Secs(30);

pub const RESOURCE_NODE: &str = "node";
pub const RESOURCE_NODE_API: &str = "node/api";
pub const RESOURCE_NODE_BUILDER: &str = "node/builder";
pub const RESOURCE_NODE_SCHEDULER: &str = "node/scheduler";
pub const RESOURCE_NODE_REPOSITORY: &str = "node/repository";
pub const RESOURCE_BUILD_SNAPSHOT: &str = "build/snapshot";
pub const RESOURCE_BUILD_METADATA: &str = "build/metadata";
pub const RESOURCE_MANIFEST_SNAPSHOT: &str = "manifest/snapshot";
pub const RESOURCE_APP_METADATA: &str = "app/metadata";
pub const RESOURCE_MANIFEST_METADATA: &str = "manifest/metadata";
pub const RESOURCE_NAMESPACE: &str = "namespace";
pub const RESOURCE_PROJECT: &str = "project";

pub const PATH_APP: &str = "app";
pub const PATH_APP_BUILD_LOG: &str = "build.log";
pub const PATH_APP_TOML_MANIFEST: &str = "app/Cargo.toml";
pub const PATH_APP_MAIN: &str = "app/src/main.rs";
pub const PATH_APP_TARGET: &str = "app/target";
pub const PATH_APP_RELEASE_BINARY: &str = "release/app";
