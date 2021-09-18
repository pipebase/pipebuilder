use pipebuilder_common::{EventType, Register, Result, WatchStream};
use tracing::{error, info};

pub struct SchedulerService {}

impl SchedulerService {
    pub fn new() -> Self {
        SchedulerService {}
    }

    pub async fn run(&self, mut register: Register) {
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
                if let Some(kv) = event.kv() {
                    let event = match event.event_type() {
                        EventType::Delete => "delete",
                        EventType::Put => "out",
                    };
                    info!(
                        "event: {}, kv: {{{}: {}}}",
                        event,
                        kv.key_str()?,
                        kv.value_str()?
                    )
                }
            }
        }
        Ok(())
    }
}
