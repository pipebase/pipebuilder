use super::Cmd;
use crate::ops::{do_builder::scan_build, print::print_records};
use pipebuilder_common::{api::client::ApiClient, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("scan")
        .about("Scan local node resource")
        .subcommands(vec![build()])
}

pub fn build() -> Cmd {
    Cmd::new("build")
        .about("Scan builds at builder")
        .args(vec![Arg::new("id")
            .short('i')
            .about("Specify builder id")
            .takes_value(true)
            .required(true)])
}

pub async fn exec_build(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let builder_id = args.value_of("id").unwrap();
    let builds = scan_build(&client, builder_id).await?;
    print_records(builds.as_slice());
    Ok(())
}
