use super::Cmd;
use crate::ops::do_node::deactivate_node;
use pipebuilder_common::{api::client::ApiClient, NodeRole, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("deactivate")
        .about("Deactivate node")
        .subcommands(vec![builder()])
}

pub fn builder() -> Cmd {
    Cmd::new("builder")
        .about("Deactivate builder given node id")
        .args(vec![Arg::new("id")
            .short('i')
            .help("Specify builder id")
            .takes_value(true)
            .required(true)])
}

pub async fn exec_builder(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let id = args.value_of("id").unwrap();
    let _ = deactivate_node(&client, NodeRole::Builder, id.to_owned()).await?;
    Ok(())
}
