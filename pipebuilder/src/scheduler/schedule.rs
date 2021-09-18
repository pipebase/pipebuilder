use pipebuilder_common::{Register, Result, WatchStream, log_event};
use tracing::{error, info};

use crate::config::SchedulerConfig;

pub struct SchedulerService {}

impl SchedulerService {
    pub fn new(_config: SchedulerConfig) -> Self {
        SchedulerService {}
    }

    pub fn run(&self, mut register: Register) {
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
            match Self::watch(stream).await {
                Ok(_) => {
                    info!("watcher {} exit ...", watcher_id)
                }
                Err(e) => {
                    error!("watcher {} exit with error {:?}", watcher_id, e)
                }
            }
        });
    }

    async fn watch(mut stream: WatchStream) -> Result<()> {
        while let Some(resp) = stream.message().await? {
            for event in resp.events() {
                log_event(event)?;
            }
        }
        Ok(())
    }
}
