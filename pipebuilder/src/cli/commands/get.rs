use super::Cmd;
use pipebuilder_common::{api::client::ApiClient, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("get")
        .about("Get resources")
        .subcommands(vec![manifest(), build()])
}

pub fn manifest() -> Cmd {
    Cmd::new("manifest")
        .about("Get manifest given namespace, app id and manifest version")
        .args(vec![
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

pub async fn exec_manifest(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    Ok(())
}

pub fn build() -> Cmd {
    Cmd::new("build")
        .about("Get build given namespace, app id and build version")
        .args(vec![
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
                .about("Specify app build version")
                .takes_value(true),
        ])
}

pub async fn exec_build(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    Ok(())
}
