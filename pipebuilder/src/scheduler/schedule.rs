use etcd_client::{EventType, WatchStream};
use flurry::HashMap;
use pipebuilder_common::{
    deserialize_event,
    grpc::schedule::{scheduler_server::Scheduler, BuilderInfo, ScheduleResponse},
    hash_distance, log_event, remove_resource, NodeRole, NodeState, Register, ScheduleDescriptor,
    ScheduleHash,
};
use std::sync::Arc;
use tonic::Response;
use tracing::{error, info, warn};

#[derive(Default)]
pub struct ScheduleManagerBuilder {}

impl ScheduleManagerBuilder {
    pub fn build(self) -> ScheduleManager {
        ScheduleManager {
            builders: Arc::new(HashMap::new()),
        }
    }
}

pub struct ScheduleManager {
    builders: Arc<HashMap<String, NodeState>>,
}

impl ScheduleManager {
    pub fn builder() -> ScheduleManagerBuilder {
        ScheduleManagerBuilder::default()
    }

    pub fn schedule(
        &self,
        schedule: ScheduleDescriptor<'_>,
        target_platform: Option<&str>,
    ) -> Option<BuilderInfo> {
        let request_key = schedule.schedule_hash();
        let builders_ref = self.builders.pin();
        let mut selected_builder_info: Option<BuilderInfo> = None;
        let mut min_hash_distance: u64 = u64::MAX;
        for (builder_key, builder) in builders_ref.iter() {
            if !builder.is_active() {
                continue;
            }
            if let Some(target_platform) = target_platform {
                if !builder.accept_target_platform(target_platform) {
                    continue;
                }
            }
            let builder_id = remove_resource::<NodeState>(builder_key).to_owned();
            let distance = hash_distance(&request_key, &builder_id);
            if distance < min_hash_distance {
                selected_builder_info = Self::builder_info(builder_id, builder);
                if selected_builder_info.is_some() {
                    min_hash_distance = distance;
                }
            }
        }
        selected_builder_info
    }

    pub fn run(&self, mut register: Register) {
        let builders = self.builders.clone();
        let _ = tokio::spawn(async move {
            let (watcher, stream) = match register.watch_nodes().await {
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
                    info!(watcher_id = watcher_id, "watcher exit ...")
                }
                Err(e) => {
                    error!(watcher_id = watcher_id, "watcher exit with error '{}'", e)
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
                if let Some((event_ty, node_key, node_state)) =
                    deserialize_event::<NodeState>(event)?
                {
                    let builders_ref = builders.pin();
                    let node_state = match event_ty {
                        EventType::Put => node_state,
                        EventType::Delete => {
                            // delete event return node_state as None, delete anyway
                            builders_ref.remove(&node_key);
                            continue;
                        }
                    };
                    let node_state = match node_state {
                        Some(node_state) => node_state,
                        None => {
                            warn!(
                                node_key = node_key.as_str(),
                                "node state undefined in watch event"
                            );
                            continue;
                        }
                    };
                    // collect builder only
                    if node_state.role == NodeRole::Builder {
                        builders_ref.insert(node_key, node_state);
                    }
                }
            }
        }
        Ok(())
    }

    fn builder_info(id: String, state: &NodeState) -> Option<BuilderInfo> {
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

pub struct SchedulerService {
    manager: ScheduleManager,
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
        let schedule = ScheduleDescriptor(namespace.as_str(), id.as_str());
        let selected_builder_info = self.manager.schedule(schedule, target_platform.as_deref());
        Ok(Response::new(ScheduleResponse {
            builder_info: selected_builder_info,
        }))
    }
}

impl SchedulerService {
    pub fn new(manager: ScheduleManager) -> Self {
        SchedulerService { manager }
    }
}
