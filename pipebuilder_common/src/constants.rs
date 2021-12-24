use crate::Period;

pub const FULL_FORMATTER: &str = "full";
pub const PRETTY_FORMATTER: &str = "pretty";
pub const JSON_FORMATTER: &str = "json";

pub const ENV_FORMATTER: &str = "PIPEBUILDER_LOG_FORMATTER";
pub const ENV_PIPEBUILDER_CONFIG_FILE: &str = "PIPEBUILDER_CONFIG_FILE";
pub const ENV_PIPEBUILDER_NODE_ID: &str = "PIPEBUILDER_NODE_ID";
pub const ENV_PIPEBUILDER_EXTERNAL_ADDR: &str = "PIPEBUILDER_EXTERNAL_ADDR";
pub const DEFAULT_NODE_HEARTBEAT_PERIOD: Period = Period::Secs(30);

pub const PATH_APP: &str = "app";
pub const PATH_APP_LOCK: &str = "app.lock";
pub const PATH_APP_BUILD_LOG: &str = "build.log";
pub const PATH_APP_TOML_MANIFEST: &str = "app/Cargo.toml";
pub const PATH_APP_MAIN: &str = "app/src/main.rs";
pub const PATH_APP_TARGET: &str = "app/target";
pub const PATH_APP_RELEASE_BINARY: &str = "release/app";
