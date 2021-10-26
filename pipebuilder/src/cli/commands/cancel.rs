use super::Cmd;
use crate::ops::do_build::cancel_build;
use clap::Arg;
use pipebuilder_common::{api::client::ApiClient, Result};

pub fn cmd() -> Cmd {
    Cmd::new("cancel")
        .about("Cancel resource")
        .subcommands(vec![build()])
}

pub fn build() -> Cmd {
    Cmd::new("build")
        .about("Cancel build given namespace, project id and build version")
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
    let _ = cancel_build(&client, namespace.to_owned(), id.to_owned(), build_version).await?;
    Ok(())
}
