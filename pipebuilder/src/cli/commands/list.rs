use super::Cmd;
use crate::ops::{
    do_build::{list_build, list_build_snapshot},
    do_manifest::list_manifest_snapshot,
    do_node::list_node_state,
    print::print_records,
};
use pipebuilder_common::{api::client::ApiClient, NodeRole, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("list").about("List resources").subcommands(vec![
        build_snapshot(),
        manifest_snapshot(),
        build(),
        node(),
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

pub fn node() -> Cmd {
    Cmd::new("node")
        .about("List node given role")
        .args(vec![Arg::new("role")
            .short('r')
            .about("Specify node role")
            .takes_value(true)])
}

pub async fn exec_node(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let role = args.value_of("role");
    let role: Option<NodeRole> = role.map(|role| role.into());
    let response = list_node_state(&client, role).await?;
    print_records(response.as_slice());
    Ok(())
}
