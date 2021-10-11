use crate::Period;

pub const ENV_PIPEBUILDER_CONFIG_FILE: &str = "PIPEBUILDER_CONFIG_FILE";
pub const ENV_PIPEBUILDER_NODE_ID: &str = "PIPEBUILDER_NODE_ID";
pub const ENV_PIPEBUILDER_EXTERNAL_ADDR: &str = "PIPEBUILDER_EXTERNAL_ADDR";
pub const DEFAULT_NODE_HEARTBEAT_PERIOD: Period = Period::Secs(30);
pub const REGISTER_KEY_API_NODE_KEY_PREFIX: &str = "/api";
pub const REGISTER_KEY_PREFIX_BUILDER: &str = "/builder";
pub const REGISTER_KEY_PREFIX_SCHEDULER: &str = "/scheduler";
pub const REGISTER_KEY_PREFIX_MANIFEST: &str = "/manifest";
pub const REGISTER_KEY_PREFIX_BUILD_SNAPSHOT: &str = "/build_snapshot";
pub const REGISTER_KEY_PREFIX_VERSION_BUILD: &str = "/build";
pub const REGISTER_KEY_PREFIX_MANIFEST_SNAPSHOT: &str = "/manifest_snapshot";