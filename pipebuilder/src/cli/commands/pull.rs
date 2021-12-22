use super::Cmd;
use crate::ops::{
    do_app::pull_app, do_build::pull_build_log, do_catalog_schema::pull_catalog_schema,
    do_catalogs::pull_catalogs, do_manifest::pull_manifest, print::print_utf8,
};
use pipebuilder_common::{api::client::ApiClient, write_file, Result};

use clap::Arg;

pub(crate) const DEFAULT_APP_DOWNLOAD_PATH: &str = "./app";

pub fn cmd() -> Cmd {
    Cmd::new("pull").about("Pull resource").subcommands(vec![
        app(),
        build_log(),
        catalogs(),
        catalog_schema(),
        manifest(),
    ])
}

pub fn manifest() -> Cmd {
    Cmd::new("manifest")
        .about("Pull manifest given namespace, project id and manifest version")
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
    let response = pull_manifest(
        &client,
        namespace.to_owned(),
        id.to_owned(),
        manifest_version,
    )
    .await?;
    print_utf8(response.buffer)
}

pub fn catalog_schema() -> Cmd {
    Cmd::new("catalog-schema")
        .about("Pull catalog schema given namespace, schema id and version")
        .args(vec![
            Arg::new("namespace")
                .short('n')
                .help("Specify namespace")
                .required(true)
                .takes_value(true),
            Arg::new("id")
                .short('i')
                .help("Specify catalog schema id")
                .required(true)
                .takes_value(true),
            Arg::new("version")
                .short('v')
                .help("Specify catalog schema version")
                .required(true)
                .takes_value(true),
        ])
}

pub async fn exec_catalog_schema(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let catalog_schema_version = args
        .value_of("version")
        .unwrap()
        .parse()
        .expect("invalid catalog schema version");
    let response = pull_catalog_schema(
        &client,
        namespace.to_owned(),
        id.to_owned(),
        catalog_schema_version,
    )
    .await?;
    print_utf8(response.buffer)
}

pub fn catalogs() -> Cmd {
    Cmd::new("catalogs")
        .about("Pull catalogs given namespace, project id and version")
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
                .help("Specify catalogs version")
                .required(true)
                .takes_value(true),
        ])
}

pub async fn exec_catalogs(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let catalogs_version = args
        .value_of("version")
        .unwrap()
        .parse()
        .expect("invalid catalogs version");
    let response = pull_catalogs(
        &client,
        namespace.to_owned(),
        id.to_owned(),
        catalogs_version,
    )
    .await?;
    print_utf8(response.buffer)
}

pub fn app() -> Cmd {
    Cmd::new("app")
        .about("Pull app binary given namespace, project id and build version")
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
            Arg::new("path")
                .short('p')
                .help("Specify app download path")
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
    let path = args.value_of("").unwrap_or(DEFAULT_APP_DOWNLOAD_PATH);
    let response = pull_app(&client, namespace.to_owned(), id.to_owned(), build_version).await?;
    let buffer = response.buffer;
    write_file(path, buffer.as_slice()).await?;
    Ok(())
}

pub fn build_log() -> Cmd {
    Cmd::new("log")
        .about("Pull build log given namespace, project id and build version")
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

pub async fn exec_build_log(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let build_version = args
        .value_of("version")
        .unwrap()
        .parse()
        .expect("invalid build version");
    let response =
        pull_build_log(&client, namespace.to_owned(), id.to_owned(), build_version).await?;
    print_utf8(response.buffer)
}
