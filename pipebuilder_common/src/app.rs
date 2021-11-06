use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// metadata for app binary (namespace, id, version)
#[derive(Deserialize, Serialize)]
pub struct AppMetadata {
    // pull count
    pub pulls: u64,
    // app binary size in byte
    pub size: usize,
    // created timestamp
    pub created: DateTime<Utc>,
}

impl AppMetadata {
    pub fn new(size: usize) -> Self {
        AppMetadata {
            pulls: 0,
            size,
            created: Utc::now(),
        }
    }
}
