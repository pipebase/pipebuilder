use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

// Latest build state per manifest id
#[derive(Default, Deserialize, Serialize)]
pub struct BuildSnapshot {
    pub latest_version: u64,
}

// A build task
pub struct Build {
    pub manifest_id: String,
    pub version: u64,
}

impl Build {
    pub fn new(manifest_id: String, version: u64) -> Self {
        Build {
            manifest_id,
            version,
        }
    }

    pub fn get_id(&self) -> String {
        self.manifest_id.to_string()
    }

    pub fn get_version(&self) -> u64 {
        self.version
    }
}
