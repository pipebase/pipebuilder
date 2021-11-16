use super::Cmd;
use crate::ops::do_node::shutdown_node;
use pipebuilder_common::{api::client::ApiClient, NodeRole, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("shutdown")
        .about("Shutdown node")
        .subcommands(vec![builder()])
}

pub fn builder() -> Cmd {
    Cmd::new("builder")
        .about("Shutdown builder given node id")
        .args(vec![Arg::new("id")
            .short('i')
            .about("Specify builder id")
            .takes_value(true)
            .required(true)])
}

pub async fn exec_builder(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let id = args.value_of("id").unwrap();
    let _ = shutdown_node(&client, NodeRole::Builder, id.to_owned()).await?;
    Ok(())
}
