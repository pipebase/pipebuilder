mod commands;
mod config;
mod ops;

use config::Config;
use pipebuilder_common::Result;
use std::{
    io::{self, Write},
    process,
};
use tracing::instrument;

#[tokio::main]
#[instrument]
async fn main() {
    let result = run().await;
    process::exit(match result {
        Ok(_) => 0,
        Err(err) => {
            let _ = writeln!(io::stderr(), "{:#?}", err);
            1
        }
    })
}

async fn run() -> Result<()> {
    let matches = clap::App::new("pbctl")
        .args(vec![clap::Arg::new("config")
            .short('c')
            .takes_value(true)
            .about("path to .pb directory")])
        .subcommands(commands::cmds())
        .get_matches();
    // TODO: parse config file
    let config = Config::default();
    // parse (action, resource) cmds
    let (action, matches) = matches.subcommand().unwrap();
    let (resource, matches) = matches.subcommand().unwrap();
    commands::exec(action, resource, &config, &matches).await
}
