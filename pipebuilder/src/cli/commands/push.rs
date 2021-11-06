use super::Cmd;
use crate::ops::{
    do_manifest::{push_manifest, read_file, validate_manifest},
    print::print_record,
};
use pipebuilder_common::{api::client::ApiClient, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("push")
        .about("Push resource")
        .subcommands(vec![manifest()])
}

pub fn manifest() -> Cmd {
    Cmd::new("manifest")
        .about("Push manifest given namespace, project id and manifest file")
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
    let response = push_manifest(&client, namespace.to_owned(), id.to_owned(), buffer).await?;
    print_record(&response);
    Ok(())
}
