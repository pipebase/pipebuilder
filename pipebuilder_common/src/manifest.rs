use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ManifestSnapshot {
    pub latest_version: u64,
}

impl ManifestSnapshot {
    pub fn new() -> Self {
        ManifestSnapshot { latest_version: 0 }
    }
}

impl Default for ManifestSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

// metadata for manifest (namespace, id, version)
#[derive(Deserialize, Serialize)]
pub struct ManifestMetadata {
    // pull count
    pub pulls: u64,
    // manifest file size in byte
    pub size: usize,
    // created timestamp
    pub created: DateTime<Utc>,
}

impl ManifestMetadata {
    pub fn new(size: usize) -> Self {
        ManifestMetadata {
            pulls: 0,
            size,
            created: Utc::now(),
        }
    }
}
