use super::Cmd;
use crate::ops::{
    do_catalog_schema::{push_catalog_schema, validate_catalog_schema},
    do_catalogs::{push_catalogs, validate_catalogs},
    do_manifest::{push_manifest, validate_manifest},
    print::print_record,
};
use pipebuilder_common::{api::client::ApiClient, read_file, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("push").about("Push resource").subcommands(vec![
        manifest(),
        catalogs(),
        catalog_schema(),
    ])
}

pub fn manifest() -> Cmd {
    Cmd::new("manifest")
        .about("Push manifest given namespace, project id and manifest file")
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
            Arg::new("file")
                .short('f')
                .help("Specify app manifest file path")
                .required(true)
                .takes_value(true),
        ])
}

pub async fn exec_manifest(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let file = args.value_of("file").unwrap();
    let buffer = read_file(file).await?;
    validate_manifest(buffer.as_slice())?;
    let response = push_manifest(&client, namespace.to_owned(), id.to_owned(), buffer).await?;
    print_record(&response);
    Ok(())
}

pub fn catalog_schema() -> Cmd {
    Cmd::new("catalog-schema")
        .about("Push catalog schema given namespace, schema id and schema file")
        .args(vec![
            Arg::new("namespace")
                .short('n')
                .help("Specify namespace")
                .required(true)
                .takes_value(true),
            Arg::new("id")
                .short('i')
                .help("Specify schema id")
                .required(true)
                .takes_value(true),
            Arg::new("file")
                .short('f')
                .help("Specify catalog schema file path")
                .required(true)
                .takes_value(true),
        ])
}

pub async fn exec_catalog_schema(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let file = args.value_of("file").unwrap();
    let buffer = read_file(file).await?;
    validate_catalog_schema(buffer.as_slice())?;
    let response =
        push_catalog_schema(&client, namespace.to_owned(), id.to_owned(), buffer).await?;
    print_record(&response);
    Ok(())
}

pub fn catalogs() -> Cmd {
    Cmd::new("catalogs")
        .about("Push catalogs given namespace, project id and catalogs file")
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
            Arg::new("file")
                .short('f')
                .help("Specify catalogs file path")
                .required(true)
                .takes_value(true),
        ])
}

pub async fn exec_catalogs(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let file = args.value_of("file").unwrap();
    let buffer = read_file(file).await?;
    validate_catalogs(&client, buffer.as_slice()).await?;
    let response = push_catalogs(&client, namespace.to_owned(), id.to_owned(), buffer).await?;
    print_record(&response);
    Ok(())
}
