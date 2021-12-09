use super::Cmd;
use crate::ops::{
    do_builder::{scan_build, scan_build_cache},
    print::print_records,
};
use pipebuilder_common::{api::client::ApiClient, Result};

use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("scan")
        .about("Scan local node resource")
        .subcommands(vec![build(), build_cache()])
}

pub fn build() -> Cmd {
    Cmd::new("build")
        .about("Scan builds at builder")
        .args(vec![Arg::new("id")
            .short('i')
            .help("Specify builder id")
            .takes_value(true)
            .required(true)])
}

pub fn build_cache() -> Cmd {
    Cmd::new("build-cache")
        .about("Scan build caches at builder")
        .args(vec![Arg::new("id")
            .short('i')
            .help("Specify builder id")
            .takes_value(true)
            .required(true)])
}

pub async fn exec_build(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let builder_id = args.value_of("id").unwrap();
    let builds = scan_build(&client, builder_id).await?;
    print_records(builds.as_slice());
    Ok(())
}

pub async fn exec_build_cache(client: ApiClient, args: &clap::ArgMatches) -> Result<()> {
    let builder_id = args.value_of("id").unwrap();
    let caches = scan_build_cache(&client, builder_id).await?;
    print_records(caches.as_slice());
    Ok(())
}
