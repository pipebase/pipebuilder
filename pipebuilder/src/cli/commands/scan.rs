use super::Cmd;
use crate::ops::{do_builder::scan_builder, print::print_records};
use pipebuilder_common::{api::client::ApiClient, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("scan")
        .about("Scan local node resource")
        .subcommands(vec![builder()])
}

pub fn builder() -> Cmd {
    Cmd::new("builder")
        .about("Scan builds at builder")
        .args(vec![Arg::new("id")
            .short('i')
            .about("Specify builder id")
            .takes_value(true)
            .required(true)])
}

pub async fn exec_builder(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let builder_id = args.value_of("id").unwrap();
    let builds = scan_builder(&client, builder_id).await?;
    print_records(builds.as_slice());
    Ok(())
}
