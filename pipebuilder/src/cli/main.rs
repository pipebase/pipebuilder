mod commands;
mod config;
mod ops;

use config::Config;
use ops::print::Printer;
use pipebuilder_common::Result;
use std::process;
use tracing::instrument;

#[tokio::main]
#[instrument]
async fn main() {
    let result = run().await;
    process::exit(match result {
        Ok(_) => 0,
        Err(_) => 1,
    })
}

async fn run() -> Result<()> {
    let mut printer = Printer::new();
    let matches = clap::App::new("pbctl")
        .args(vec![clap::Arg::new("config")
            .short('c')
            .takes_value(true)
            .help("path to config file, default ~/.pb/config")])
        .subcommands(commands::cmds())
        .get_matches();
    let config_path = matches.value_of("config");
    let config = Config::parse_or_default(config_path).await;
    let api_config = config.api;
    let api_client = api_config.into();
    // parse (action, resource) cmds
    let (action, matches) = matches.subcommand().unwrap();
    let (resource, matches) = matches.subcommand().unwrap();
    match commands::exec(action, resource, api_client, matches).await {
        Ok(_) => Ok(()),
        Err(err) => {
            let _ = printer.error(&err);
            Err(err)
        }
    }
}
