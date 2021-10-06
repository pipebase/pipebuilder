use crate::config::Config;
use pipebuilder_common::Result;

pub(crate) mod create;
pub(crate) mod get;

pub type Cmd = clap::App<'static>;

pub fn cmds() -> Vec<Cmd> {
    vec![create::cmd(), get::cmd()]
}

// exec given cmds (action, resource), config and args
pub async fn exec(
    action: &str,
    resource: &str,
    config: &Config,
    args: &clap::ArgMatches,
) -> Result<()> {
    match (action, resource) {
        ("get", "manifest") => get::exec_manifest(config, args).await,
        ("get", "build") => get::exec_build(config, args).await,
        ("create", "manifest") => create::exec_manifest(config, args).await,
        ("create", "build") => create::exec_build(config, args).await,
        _ => unreachable!("unknown cmd ({}, {})", action, resource),
    }
}
