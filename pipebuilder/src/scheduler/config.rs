use pipebuilder_common::BaseConfig;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SchedulerConfig {}

#[derive(Deserialize)]
pub struct Config {
    pub base: BaseConfig,
    // pub scheduler: SchedulerConfig,
}
