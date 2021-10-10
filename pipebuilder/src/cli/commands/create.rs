use super::Cmd;
use crate::ops::{
    do_build,
    do_manifest::{put_manifest, read_file, validate_manifest},
    print::print_record,
};
use pipebuilder_common::{api::client::ApiClient, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("create")
        .about("Create resource")
        .subcommands(vec![manifest(), build()])
}

pub fn manifest() -> Cmd {
    Cmd::new("manifest")
        .about("Create manifest given namespace, app id and manifest file")
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
            Arg::new("file")
                .short('f')
                .about("Specify app manifest file path")
                .required(true)
                .takes_value(true),
        ])
}

pub async fn exec_manifest(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let namespace = args.value_of("namespace").unwrap();
    let id = args.value_of("id").unwrap();
    let file = args.value_of("file").unwrap();
    validate_manifest(&client, file)?;
    let buffer = read_file(file)?;
    let response = put_manifest(&client, namespace.to_owned(), id.to_owned(), buffer).await?;
    print_record(&response);
    Ok(())
}

pub fn build() -> Cmd {
    Cmd::new("build")
        .about("Create build given namespace and app id")
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
                .about("Specify manifest version")
                .required(true)
                .takes_value(true),
            Arg::new("target")
                .short('t')
                .about("Specify target platform")
                .required(true)
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
    let target_platform = args.value_of("target_platform").unwrap();
    let response = do_build::build(
        &client,
        namespace.to_owned(),
        id.to_owned(),
        manifest_version,
        target_platform.to_owned(),
    )
    .await?;
    print_record(&response);
    Ok(())
}
