use super::Cmd;
use crate::ops::{
    do_app::list_app_metadata,
    do_build::{list_build_metadata, list_build_snapshot},
    do_manifest::{list_manifest_metadata, list_manifest_snapshot},
    do_namespace::list_namespace,
    do_node::list_node_state,
    do_project::list_project,
    print::print_records,
};
use pipebuilder_common::{api::client::ApiClient, NodeRole, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("list").about("List resource").subcommands(vec![
        app(),
        build(),
        manifest(),
        node(),
        namespace(),
        project(),
    ])
}

pub fn build() -> Cmd {
    Cmd::new("build")
        .about("List build metadata given namespace and project id")
        .args(vec![
            Arg::new("namespace")
                .short('n')
                .about("Specify namespace")
                .required(true)
                .takes_value(true),
            Arg::new("id")
                .short('i')
                .about("Specify project id")
                .takes_value(true),
            Arg::new("snapshot")
                .short('s')
                .about("Specify build snapshot per project id returned"),
        ])
}

async fn exec_build_snapshot(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let response = list_build_snapshot(&client, namespace.to_owned()).await?;
    print_records(response.as_slice());
    Ok(())
}

async fn exec_build_metadata(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").map(|id| id.to_owned());
    let response = list_build_metadata(&client, namespace.to_owned(), id.to_owned()).await?;
    print_records(response.as_slice());
    Ok(())
}

pub async fn exec_build(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let is_snapshot = args.is_present("snapshot");
    if is_snapshot {
        return exec_build_snapshot(client, args).await;
    }
    return exec_build_metadata(client, args).await;
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

pub fn app() -> Cmd {
    Cmd::new("app").about("List app metadata").args(vec![
        Arg::new("namespace")
            .short('n')
            .about("Specify namespace")
            .required(true)
            .takes_value(true),
        Arg::new("id")
            .short('i')
            .about("Specify project id")
            .takes_value(true),
    ])
}

pub async fn exec_app(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id");
    let id = id.map(|id| id.to_owned());
    let response = list_app_metadata(&client, namespace.to_owned(), id).await?;
    print_records(response.as_slice());
    Ok(())
}

pub fn manifest() -> Cmd {
    Cmd::new("manifest")
        .about("List manifest metadata")
        .args(vec![
            Arg::new("namespace")
                .short('n')
                .about("Specify namespace")
                .required(true)
                .takes_value(true),
            Arg::new("id")
                .short('i')
                .about("Specify project id")
                .takes_value(true),
            Arg::new("snapshot")
                .short('s')
                .about("Specify manifest snapshot per project id returned"),
        ])
}

async fn exec_manifest_snapshot(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let response = list_manifest_snapshot(&client, namespace.to_owned()).await?;
    print_records(response.as_slice());
    Ok(())
}

async fn exec_manifest_metadata(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id");
    let id = id.map(|id| id.to_owned());
    let response = list_manifest_metadata(&client, namespace.to_owned(), id).await?;
    print_records(response.as_slice());
    Ok(())
}

pub async fn exec_manifest(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let is_snapshot = args.is_present("snapshot");
    if is_snapshot {
        return exec_manifest_snapshot(client, args).await;
    }
    return exec_manifest_metadata(client, args).await;
}

pub fn namespace() -> Cmd {
    Cmd::new("namespace").about("List namespace")
}

pub async fn exec_namespace(client: ApiClient, _args: &clap::ArgMatches) -> Result<()> {
    let response = list_namespace(&client).await?;
    print_records(response.as_slice());
    Ok(())
}

pub fn project() -> Cmd {
    Cmd::new("project")
        .about("list project given namespace id")
        .args(vec![Arg::new("namespace")
            .short('n')
            .about("Specify namespace id")
            .required(true)
            .takes_value(true)])
}

pub async fn exec_project(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let response = list_project(&client, namespace).await?;
    print_records(response.as_slice());
    Ok(())
}
