use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub enum BuildStatus {
    // register version build
    Create,
    // pull manifest
    Pull,
    // validate manifest
    Validate,
    // initialize rust app
    // restore from previous build or create a new project
    Initialize,
    // generate rust code
    Generate,
    // cargo build
    Build,
    // store compiled results
    Store,
    // publish app binary
    Publish,
    // succeed all steps
    Done,
    // build failed
    Fail,
}

// Build state per (build_id, version)
#[derive(Deserialize, Serialize)]
pub struct VersionBuild {
    // build status
    pub status: BuildStatus,
    // timestamp
    pub timestamp: DateTime<Utc>,
    // message
    pub message: Option<String>,
}

impl VersionBuild {
    pub fn new(status: BuildStatus, timestamp: DateTime<Utc>, message: Option<String>) -> Self {
        VersionBuild {
            status,
            timestamp,
            message,
        }
    }
}

// Latest build state per manifest url
#[derive(Deserialize, Serialize)]
pub struct BuildSnapshot {
    pub id: Uuid,
    pub latest_version: u64,
}

impl BuildSnapshot {
    pub fn new() -> Self {
        BuildSnapshot {
            id: Uuid::new_v4(),
            latest_version: 0,
        }
    }
}

// A build task
pub struct Build {
    pub manifest_url: String,
    pub id: Uuid,
    pub version: u64,
}

impl Build {
    pub fn new(manifest_url: String, id: Uuid, version: u64) -> Self {
        Build {
            manifest_url,
            id,
            version,
        }
    }

    pub fn get_string_id(&self) -> String {
        self.id.to_string()
    }

    pub fn get_version(&self) -> u64 {
        self.version
    }
}
