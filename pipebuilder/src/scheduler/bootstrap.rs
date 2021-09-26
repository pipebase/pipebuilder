use crate::{config::SchedulerConfig, schedule::SchedulerService};
use pipebuilder_common::{grpc::schedule::scheduler_server::SchedulerServer, Register};

pub fn bootstrap(config: SchedulerConfig, register: Register) -> SchedulerServer<SchedulerService> {
    let scheduler = SchedulerService::new(config);
    scheduler.run(register);
    SchedulerServer::new(scheduler)
}
