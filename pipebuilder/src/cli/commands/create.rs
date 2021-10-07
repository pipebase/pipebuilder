use super::Cmd;
use pipebuilder_common::{api::client::ApiClient, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("create")
        .about("Create resource")
        .subcommands(vec![manifest(), build()])
}

pub fn manifest() -> Cmd {
    Cmd::new("manifest")
        .about("Create manifest given namespace, app id and manifest file")
        .args(vec![
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

pub async fn exec_manifest(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    Ok(())
}

pub fn build() -> Cmd {
    Cmd::new("build")
        .about("Create build given namespace and app id")
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

pub async fn exec_build(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    Ok(())
}
