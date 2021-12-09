use super::Cmd;
use crate::ops::{do_build::get_build_metadata, print::print_records};
use pipebuilder_common::{api::client::ApiClient, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("get")
        .about("Get resource")
        .subcommands(vec![build()])
}

pub fn build() -> Cmd {
    Cmd::new("build")
        .about("Get build metadata given namespace, project id and build version")
        .args(vec![
            Arg::new("namespace")
                .short('n')
                .help("Specify namespace")
                .takes_value(true)
                .required(true),
            Arg::new("id")
                .short('i')
                .help("Specify project id")
                .takes_value(true)
                .required(true),
            Arg::new("version")
                .short('v')
                .help("Specify app build version")
                .takes_value(true)
                .required(true),
        ])
}

pub async fn exec_build(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let build_version = args
        .value_of("version")
        .unwrap()
        .parse()
        .expect("invalid build version");
    let response =
        get_build_metadata(&client, namespace.to_owned(), id.to_owned(), build_version).await?;
    let responses = vec![response];
    print_records(responses.as_slice());
    Ok(())
}
