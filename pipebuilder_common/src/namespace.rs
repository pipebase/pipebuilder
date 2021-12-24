use crate::{Resource, ResourceType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Namespace {
    pub created: DateTime<Utc>,
}

impl Namespace {
    pub fn new() -> Self {
        let created = Utc::now();
        Namespace { created }
    }
}

impl Default for Namespace {
    fn default() -> Self {
        Self::new()
    }
}

impl Resource for Namespace {
    fn ty() -> ResourceType {
        ResourceType::Namespace
    }
}
