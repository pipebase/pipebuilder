use crate::{Register, Result};

use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
pub struct LeaseConfig {
    // https://etcd.io/docs/v3.5/learning/api/#obtaining-leases
    // ttl in seconds
    ttl: u64,
}

pub struct LeaseService {
    lease_id: i64,
    ttl: u64,
    register: Register,
}

impl LeaseService {
    pub async fn new(config: LeaseConfig, mut register: Register) -> Result<Self> {
        let ttl = config.ttl;
        let resp = register.lease_grant(ttl as i64).await?;
        let lease_id = resp.id();
        Ok(LeaseService {
            lease_id,
            ttl,
            register,
        })
    }

    pub fn get_lease_id(&self) -> i64 {
        self.lease_id
    }

    pub async fn run(&mut self) -> Result<()> {
        // refresh every ttl / 2 second
        let mut refresh_interval = tokio::time::interval(Duration::from_secs(self.ttl / 2));
        loop {
            refresh_interval.tick().await;
            self.register.lease_keep_alive(self.lease_id).await?;
        }
    }
}
