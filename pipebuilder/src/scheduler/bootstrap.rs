use crate::{config::SchedulerConfig, schedule::SchedulerService};
use pipebuilder_common::Register;

pub fn bootstrap(config: SchedulerConfig, register: Register) -> SchedulerService {
    let scheduler = SchedulerService::new(config);
    scheduler.run(register);
    scheduler
}
