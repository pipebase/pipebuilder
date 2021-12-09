use super::Cmd;
use crate::ops::{
    do_build,
    do_namespace::create_namespace,
    do_project::create_project,
    print::{print_record, print_records},
};
use pipebuilder_common::{api::client::ApiClient, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("create")
        .about("Create resource")
        .subcommands(vec![build(), namespace(), project()])
}

pub fn build() -> Cmd {
    Cmd::new("build")
        .about("Create build given namespace and project id")
        .args(vec![
            Arg::new("namespace")
                .short('n')
                .help("Specify namespace")
                .required(true)
                .takes_value(true),
            Arg::new("id")
                .short('i')
                .help("Specify project id")
                .required(true)
                .takes_value(true),
            Arg::new("version")
                .short('v')
                .help("Specify manifest version")
                .required(true)
                .takes_value(true),
            Arg::new("target-platform")
                .short('t')
                .help("Specify target platform, checkout https://doc.rust-lang.org/nightly/rustc/platform-support.html")
                .takes_value(true),
        ])
}

pub async fn exec_build(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let manifest_version = args
        .value_of("version")
        .unwrap()
        .parse()
        .expect("invalid manifest version");
    let target_platform = args.value_of("target-platform").map(String::from);
    let response = do_build::build(
        &client,
        namespace.to_owned(),
        id.to_owned(),
        manifest_version,
        target_platform,
    )
    .await?;
    print_record(&response);
    Ok(())
}

pub fn namespace() -> Cmd {
    Cmd::new("namespace")
        .about("Create namespace given namespace id")
        .args(vec![Arg::new("id")
            .short('i')
            .help("Specify namespace id")
            .required(true)
            .takes_value(true)])
}

pub async fn exec_namespace(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let id = args.value_of("id").unwrap();
    let namespace = create_namespace(&client, id.to_owned()).await?;
    let namespaces = vec![namespace];
    print_records(namespaces.as_slice());
    Ok(())
}

pub fn project() -> Cmd {
    Cmd::new("project")
        .about("Create project given namespace and project id")
        .args(vec![
            Arg::new("namespace")
                .short('n')
                .help("Specify namespace")
                .required(true)
                .takes_value(true),
            Arg::new("id")
                .short('i')
                .help("Specify project id")
                .required(true)
                .takes_value(true),
        ])
}

pub async fn exec_project(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let project = create_project(&client, namespace.to_owned(), id.to_owned()).await?;
    let projects = vec![project];
    print_records(projects.as_slice());
    Ok(())
}
