use super::Cmd;
use crate::ops::{do_app::delete_app, do_build::delete_build, do_manifest::delete_manifest};
use pipebuilder_common::{api::client::ApiClient, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("delete")
        .about("Delete resource")
        .subcommands(vec![manifest(), build(), app()])
}

pub fn manifest() -> Cmd {
    Cmd::new("manifest")
        .about("Delete manifest given namespace, project id and manifest version")
        .args(vec![
            Arg::new("namespace")
                .short('n')
                .about("Specify namespace")
                .required(true)
                .takes_value(true),
            Arg::new("id")
                .short('i')
                .about("Specify project id")
                .required(true)
                .takes_value(true),
            Arg::new("version")
                .short('v')
                .about("Specify app manifest version")
                .required(true)
                .takes_value(true),
        ])
}

pub async fn exec_manifest(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let manifest_version = args
        .value_of("version")
        .unwrap()
        .parse()
        .expect("invalid manifest version");
    delete_manifest(
        &client,
        namespace.to_owned(),
        id.to_owned(),
        manifest_version,
    )
    .await
}

pub fn build() -> Cmd {
    Cmd::new("build")
        .about("Delete build given namespace, project id and build version")
        .args(vec![
            Arg::new("namespace")
                .short('n')
                .about("Specify namespace")
                .takes_value(true)
                .required(true),
            Arg::new("id")
                .short('i')
                .about("Specify project id")
                .takes_value(true)
                .required(true),
            Arg::new("version")
                .short('v')
                .about("Specify app build version")
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
    delete_build(&client, namespace.to_owned(), id.to_owned(), build_version).await
}

pub fn app() -> Cmd {
    Cmd::new("app")
        .about("Delete app binary given namespace, project id and build version")
        .args(vec![
            Arg::new("namespace")
                .short('n')
                .about("Specify namespace")
                .takes_value(true)
                .required(true),
            Arg::new("id")
                .short('i')
                .about("Specify project id")
                .takes_value(true)
                .required(true),
            Arg::new("version")
                .short('v')
                .about("Specify app build version")
                .takes_value(true)
                .required(true),
            Arg::new("path")
                .short('p')
                .about("Specify app download path")
                .takes_value(true),
        ])
}

pub async fn exec_app(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let build_version = args
        .value_of("version")
        .unwrap()
        .parse()
        .expect("invalid build version");
    delete_app(&client, namespace.to_owned(), id.to_owned(), build_version).await
}
