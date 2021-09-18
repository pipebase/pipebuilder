use etcd_client::{EventType, WatchStream};
use flurry::HashMap;
use pipebuilder_common::{deserialize_event, log_event, NodeState, Register, Result};
use std::sync::Arc;
use tracing::{error, info};

use crate::config::SchedulerConfig;

pub struct SchedulerService {
    builders: Arc<HashMap<String, NodeState>>,
}

impl SchedulerService {
    pub fn new(_config: SchedulerConfig) -> Self {
        SchedulerService {
            builders: Arc::new(HashMap::new()),
        }
    }

    pub fn run(&self, mut register: Register) {
        let builders = self.builders.clone();
        let _ = tokio::spawn(async move {
            let (watcher, stream) = match register.watch_builders().await {
                Ok((watcher, stream)) => (watcher, stream),
                Err(e) => {
                    error!("scheduler service watch failed: {:?}", e);
                    return;
                }
            };
            let watcher_id = watcher.watch_id();
            info!("create watcher {}", watcher_id);
            match Self::watch(stream, builders.clone()).await {
                Ok(_) => {
                    info!("watcher {} exit ...", watcher_id)
                }
                Err(e) => {
                    error!("watcher {} exit with error {:?}", watcher_id, e)
                }
            };
            // cleanup if stop watching
            builders.pin().clear();
        });
    }

    // watch builder events
    async fn watch(
        mut stream: WatchStream,
        builders: Arc<HashMap<String, NodeState>>,
    ) -> Result<()> {
        while let Some(resp) = stream.message().await? {
            for event in resp.events() {
                log_event(event)?;
                if let Some((event_ty, key, node)) = deserialize_event(event)? {
                    let builders_ref = builders.pin();
                    match event_ty {
                        EventType::Put => builders_ref.insert(key, node),
                        EventType::Delete => builders_ref.remove(&key),
                    };
                }
            }
        }
        Ok(())
    }
}
