use etcd_client::{EventType, WatchStream};
use flurry::HashMap;
use pipebuilder_common::{
    deserialize_event,
    grpc::schedule::{scheduler_server::Scheduler, BuilderInfo, ScheduleResponse},
    hash_distance, log_event, NodeState, Register,
};
use std::sync::Arc;
use tonic::Response;
use tracing::{error, info, log::warn};

use crate::config::SchedulerConfig;

pub struct SchedulerService {
    builders: Arc<HashMap<String, NodeState>>,
}

#[tonic::async_trait]
impl Scheduler for SchedulerService {
    async fn schedule(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::schedule::ScheduleRequest>,
    ) -> Result<tonic::Response<pipebuilder_common::grpc::schedule::ScheduleResponse>, tonic::Status>
    {
        // select builder using consistent hash, build of same app (namespace, id) landed on same builder for compilcation cache hit
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        let target_platform = request.target_platform;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            "schedule build"
        );
        let request_key = format!("{}/{}", namespace, id);
        let builders_ref = self.builders.pin();
        let mut selected_builder_info: Option<BuilderInfo> = None;
        let mut min_hash_distance: u64 = u64::MAX;
        for builder in builders_ref.values() {
            if !builder.is_active() {
                continue;
            }
            if let Some(ref target_platform) = target_platform {
                if !builder.accept_target_platform(target_platform) {
                    continue;
                }
            }
            let builder_key = builder.id.to_owned();
            let distance = hash_distance(&request_key, &builder_key);
            if distance < min_hash_distance {
                selected_builder_info = Self::builder_info(builder);
                if selected_builder_info.is_some() {
                    min_hash_distance = distance;
                }
            }
        }
        Ok(Response::new(ScheduleResponse {
            builder_info: selected_builder_info,
        }))
    }
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
                    error!("scheduler service watch fail, error '{}'", e);
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
                    error!("watcher {} exit with error '{}'", watcher_id, e)
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
    ) -> pipebuilder_common::Result<()> {
        while let Some(resp) = stream.message().await? {
            for event in resp.events() {
                log_event(event)?;
                if let Some((event_ty, key, node)) = deserialize_event::<NodeState>(event)? {
                    let builders_ref = builders.pin();
                    let node = match event_ty {
                        EventType::Put => node,
                        EventType::Delete => {
                            builders_ref.remove(&key);
                            continue;
                        }
                    };
                    match node {
                        Some(node) => {
                            builders_ref.insert(key, node);
                        }
                        None => warn!("node state undefined for {}", key),
                    };
                }
            }
        }
        Ok(())
    }

    fn builder_info(state: &NodeState) -> Option<BuilderInfo> {
        let id = state.id.to_owned();
        let address = state.external_address.to_owned();
        state
            .get_support_target_platform()
            .map(|target_platform| BuilderInfo {
                id,
                address,
                target_platform,
            })
    }
}
