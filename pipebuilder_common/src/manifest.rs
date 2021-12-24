use crate::{BlobResource, Resource, ResourceType, Snapshot};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct ManifestSnapshot {
    pub latest_version: u64,
}

impl Snapshot for ManifestSnapshot {
    fn incr_version(&mut self) {
        self.latest_version += 1;
    }
}

impl Resource for ManifestSnapshot {
    fn ty() -> ResourceType {
        ResourceType::ManifestSnapshot
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

impl BlobResource for ManifestMetadata {
    fn new(size: usize) -> Self {
        ManifestMetadata {
            pulls: 0,
            size,
            created: Utc::now(),
        }
    }

    fn incr_usage(&mut self) {
        self.pulls += 1
    }
}

impl Resource for ManifestMetadata {
    fn ty() -> ResourceType {
        ResourceType::ManifestMetadata
    }
}
