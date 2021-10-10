use super::Cmd;
use crate::ops::{
    do_build::{list_build, list_build_snapshot},
    do_manifest::list_manifest_snapshot,
    print::print_records,
};
use pipebuilder_common::{api::client::ApiClient, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("list").about("List resources").subcommands(vec![
        build_snapshot(),
        manifest_snapshot(),
        build(),
    ])
}

pub fn build_snapshot() -> Cmd {
    Cmd::new("build-snapshot")
        .about("List build snapshot given namespace")
        .args(vec![Arg::new("namespace")
            .short('n')
            .about("Specify namespace")
            .required(true)
            .takes_value(true)])
}

pub async fn exec_build_snapshot(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let response = list_build_snapshot(&client, namespace.to_owned()).await?;
    print_records(response.as_slice());
    Ok(())
}

pub fn manifest_snapshot() -> Cmd {
    Cmd::new("manifest-snapshot")
        .about("List manifest snapshot given namespace")
        .args(vec![Arg::new("namespace")
            .short('n')
            .about("Specify namespace")
            .required(true)
            .takes_value(true)])
}

pub async fn exec_manifest_snapshot(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let response = list_manifest_snapshot(&client, namespace.to_owned()).await?;
    print_records(response.as_slice());
    Ok(())
}

pub fn build() -> Cmd {
    Cmd::new("build")
        .about("List build history given namespace and app id")
        .args(vec![
            Arg::new("namespace")
                .short('n')
                .about("Specify namespace")
                .required(true)
                .takes_value(true),
            Arg::new("id")
                .short('i')
                .about("Specify app id")
                .required(true)
                .takes_value(true),
        ])
}

pub async fn exec_build(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let response = list_build(&client, namespace.to_owned(), id.to_owned()).await?;
    print_records(response.as_slice());
    Ok(())
}
