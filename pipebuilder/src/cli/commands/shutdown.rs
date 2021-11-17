use super::Cmd;
use crate::ops::do_node::shutdown_node;
use pipebuilder_common::{api::client::ApiClient, NodeRole, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("shutdown")
        .about("Shutdown node")
        .subcommands(vec![builder(), repository(), scheduler()])
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

pub fn scheduler() -> Cmd {
    Cmd::new("scheduler")
        .about("Shutdown scheduler given node id")
        .args(vec![Arg::new("id")
            .short('i')
            .about("Specify scheduler id")
            .takes_value(true)
            .required(true)])
}

pub async fn exec_scheduler(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let id = args.value_of("id").unwrap();
    let _ = shutdown_node(&client, NodeRole::Scheduler, id.to_owned()).await?;
    Ok(())
}

pub fn repository() -> Cmd {
    Cmd::new("repository")
        .about("Shutdown repository given node id")
        .args(vec![Arg::new("id")
            .short('i')
            .about("Specify repository id")
            .takes_value(true)
            .required(true)])
}

pub async fn exec_repository(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let id = args.value_of("id").unwrap();
    let _ = shutdown_node(&client, NodeRole::Repository, id.to_owned()).await?;
    Ok(())
}
