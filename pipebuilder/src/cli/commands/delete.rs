use super::Cmd;
use crate::ops::{
    do_app::{delete_app, delete_app_all},
    do_build::{delete_build, delete_build_all, delete_build_cache},
    do_manifest::{delete_manifest, delete_manifest_all},
    do_namespace::delete_namespace,
    do_project::delete_project,
};
use pipebuilder_common::{api::client::ApiClient, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("delete")
        .about("Delete resource")
        .subcommands(vec![
            manifest(),
            build(),
            app(),
            project(),
            namespace(),
            build_cache(),
        ])
}

pub fn manifest() -> Cmd {
    Cmd::new("manifest")
        .about("Delete manifest given namespace, project id and manifest version, if no version provide, all manifest deleted")
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
                .help("Specify app manifest version")
                .takes_value(true),
        ])
}

pub async fn exec_manifest(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let manifest_version: u64 = match args.value_of("version") {
        Some(version) => version.parse().expect("invalid manifest version"),
        None => return delete_manifest_all(&client, namespace.to_owned(), id.to_owned()).await,
    };
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
        .about("Delete build given namespace, project id and build version, if no build version provide, all build deleted")
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
                .takes_value(true),
        ])
}

pub async fn exec_build(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let build_version: u64 = match args.value_of("version") {
        Some(version) => version.parse().expect("invalid build version"),
        None => return delete_build_all(&client, namespace.to_owned(), id.to_owned()).await,
    };
    delete_build(&client, namespace.to_owned(), id.to_owned(), build_version).await
}

pub fn app() -> Cmd {
    Cmd::new("app")
        .about("Delete app binary given namespace, project id and build version, if no build version provide, all app deleted")
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
                .takes_value(true),
        ])
}

pub async fn exec_app(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let build_version: u64 = match args.value_of("version") {
        Some(version) => version.parse().expect("invalid build version"),
        None => return delete_app_all(&client, namespace.to_owned(), id.to_owned()).await,
    };
    delete_app(&client, namespace.to_owned(), id.to_owned(), build_version).await
}

pub fn namespace() -> Cmd {
    Cmd::new("namespace")
        .about("Delete namespace given namespace id")
        .args(vec![Arg::new("id")
            .short('i')
            .help("Specify namespace id")
            .required(true)
            .takes_value(true)])
}

pub async fn exec_namespace(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let id = args.value_of("id").unwrap();
    delete_namespace(&client, id.to_owned()).await
}

pub fn project() -> Cmd {
    Cmd::new("project")
        .about("Delete project given namespace, project id")
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
    delete_project(&client, namespace.to_owned(), id.to_owned()).await
}

pub fn build_cache() -> Cmd {
    Cmd::new("build-cache")
        .about("Delete build-cache given builder id, namespace, project id and target platform")
        .args(vec![
            Arg::new("builder")
                .short('b')
                .help("Specify builder id")
                .takes_value(true)
                .required(true),
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
            Arg::new("target-platform")
                .short('t')
                .help("Specify target platform, checkout https://doc.rust-lang.org/nightly/rustc/platform-support.html")
                .takes_value(true),
        ])
}

pub async fn exec_build_cache(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let builder_id = args.value_of("builder").unwrap();
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let target_platform = args.value_of("target-platform").unwrap();
    delete_build_cache(
        &client,
        builder_id.to_owned(),
        namespace.to_owned(),
        id.to_owned(),
        target_platform.to_owned(),
    )
    .await
}
