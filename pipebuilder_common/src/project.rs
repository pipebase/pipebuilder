use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Project {
    pub created: DateTime<Utc>,
}

impl Project {
    pub fn new() -> Self {
        let created = Utc::now();
        Project { created }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}
