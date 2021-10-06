use super::Cmd;
use crate::config::Config;
use pipebuilder_common::Result;

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("list")
        .about("List resources")
        .subcommands(vec![snapshot(), build()])
}

pub fn snapshot() -> Cmd {
    Cmd::new("snapshot")
        .about("List build or manifest snapshot given namespace")
        .args(vec![
            Arg::new("namespace")
                .short('n')
                .about("Specify namespace")
                .takes_value(true),
            Arg::new("build").short('b').about("List build snapshot"),
            Arg::new("manifest")
                .short('m')
                .about("List manifest snapshot"),
        ])
}

pub async fn exec_snapshot(config: &Config, args: &clap::ArgMatches) -> Result<()> {
    Ok(())
}

pub fn build() -> Cmd {
    Cmd::new("build")
        .about("List build history given namespace and app id")
        .args(vec![
            Arg::new("namespace")
                .short('n')
                .about("Specify namespace")
                .takes_value(true),
            Arg::new("id")
                .short('i')
                .about("Specify app id")
                .takes_value(true),
        ])
}

pub async fn exec_build(config: &Config, args: &clap::ArgMatches) -> Result<()> {
    Ok(())
}
