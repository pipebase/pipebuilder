use crate::{
    config::SchedulerConfig,
    schedule::{ScheduleManager, SchedulerService},
};
use pipebuilder_common::Register;

pub fn bootstrap(_config: SchedulerConfig, register: Register) -> SchedulerService {
    let manager = ScheduleManager::builder().build();
    // start builder watcher
    manager.run(register);
    SchedulerService::new(manager)
}
