use super::Cmd;
use crate::config::Config;
use pipebuilder_common::Result;

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("create")
        .about("Create resource")
        .subcommands(vec![manifest(), build()])
}

pub fn manifest() -> Cmd {
    Cmd::new("manifest").about("Create manifest").args(vec![
        Arg::new("namespace")
            .short('n')
            .about("Specify namespace")
            .takes_value(true),
        Arg::new("id")
            .short('i')
            .about("Specify app id")
            .takes_value(true),
        Arg::new("file")
            .short('f')
            .about("Specify app manifest file path")
            .takes_value(true),
    ])
}

pub async fn exec_manifest(config: &Config, args: &clap::ArgMatches) -> Result<()> {
    Ok(())
}

pub fn build() -> Cmd {
    Cmd::new("build").about("Create build").args(vec![
        Arg::new("namespace")
            .short('n')
            .about("Specify namespace")
            .takes_value(true),
        Arg::new("id")
            .short('i')
            .about("Specify app id")
            .takes_value(true),
        Arg::new("version")
            .short('v')
            .about("Specify app manifest version")
            .takes_value(true),
    ])
}

pub async fn exec_build(config: &Config, args: &clap::ArgMatches) -> Result<()> {
    Ok(())
}
