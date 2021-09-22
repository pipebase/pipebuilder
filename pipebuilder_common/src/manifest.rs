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
