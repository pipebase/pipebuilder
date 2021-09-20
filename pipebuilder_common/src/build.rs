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
}

impl VersionBuild {
    pub fn new(status: BuildStatus, timestamp: DateTime<Utc>) -> Self {
        VersionBuild { status, timestamp }
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
