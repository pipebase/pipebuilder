use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ManifestSnapshot {
    pub name: String,
    pub url: String,
    pub latest_version: u64,
}

impl ManifestSnapshot {
    pub fn new(name: String, url: String) -> Self {
        ManifestSnapshot {
            name,
            url,
            latest_version: 0,
        }
    }
}
