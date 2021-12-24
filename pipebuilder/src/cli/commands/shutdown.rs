use super::Cmd;
use crate::ops::do_node::shutdown_node;
use pipebuilder_common::{api::client::ApiClient, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("shutdown")
        .about("Shutdown resources")
        .subcommands(vec![node()])
}

pub fn node() -> Cmd {
    Cmd::new("node")
        .about("Shutdown node given id")
        .args(vec![Arg::new("id")
            .short('i')
            .help("Specify node id")
            .takes_value(true)
            .required(true)])
}

pub async fn exec_node(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let id = args.value_of("id").unwrap();
    let _ = shutdown_node(&client, id.to_owned()).await?;
    Ok(())
}
