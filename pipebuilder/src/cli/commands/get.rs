use super::Cmd;
use crate::ops::{
    do_build::get_build,
    do_manifest::get_manifest,
    print::{print_record, print_utf8},
};
use pipebuilder_common::{api::client::ApiClient, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("get")
        .about("Get resources")
        .subcommands(vec![manifest(), build()])
}

pub fn manifest() -> Cmd {
    Cmd::new("manifest")
        .about("Get manifest given namespace, app id and manifest version")
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
    let response = get_manifest(
        &client,
        namespace.to_owned(),
        id.to_owned(),
        manifest_version,
    )
    .await?;
    print_utf8(response.buffer)?;
    Ok(())
}

pub fn build() -> Cmd {
    Cmd::new("build")
        .about("Get build given namespace, app id and build version")
        .args(vec![
            Arg::new("namespace")
                .short('n')
                .about("Specify namespace")
                .takes_value(true),
            Arg::new("id")
                .short('i')
                .about("Specify app id")
                .takes_value(true),
            Arg::new("version")
                .short('v')
                .about("Specify app build version")
                .takes_value(true),
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
    let response = get_build(&client, namespace.to_owned(), id.to_owned(), build_version).await?;
    print_record(&response);
    Ok(())
}
