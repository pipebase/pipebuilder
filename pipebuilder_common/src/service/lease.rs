use crate::Register;

use serde::Deserialize;
use std::time::Duration;
use tracing::error;

#[derive(Deserialize)]
pub struct LeaseConfig {
    // https://etcd.io/docs/v3.5/learning/api/#obtaining-leases
    // ttl in seconds
    pub ttl: u64,
}

pub struct LeaseService {
    lease_id: i64,
    ttl: u64,
}

impl LeaseService {
    pub fn new(config: LeaseConfig, lease_id: i64) -> Self {
        let ttl = config.ttl;
        LeaseService { lease_id, ttl }
    }

    pub fn get_lease_id(&self) -> i64 {
        self.lease_id
    }

    pub fn run(&self, mut register: Register) {
        // refresh every ttl / 2 second
        let mut refresh_interval = tokio::time::interval(Duration::from_secs(self.ttl / 2));
        let lease_id = self.lease_id;
        let _ = tokio::spawn(async move {
            loop {
                refresh_interval.tick().await;
                match register.lease_keep_alive(lease_id).await {
                    Ok(_) => continue,
                    Err(e) => {
                        error!("lease keep alive error {:?}, stop lease renew", e);
                        break;
                    }
                }
            }
        });
    }
}
