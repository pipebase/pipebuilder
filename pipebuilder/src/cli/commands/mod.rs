use pipebuilder_common::{api::client::ApiClient, Result};

pub(crate) mod cancel;
pub(crate) mod create;
pub(crate) mod get;
pub(crate) mod list;
pub(crate) mod scan;

pub type Cmd = clap::App<'static>;

pub fn cmds() -> Vec<Cmd> {
    vec![
        create::cmd(),
        get::cmd(),
        list::cmd(),
        cancel::cmd(),
        scan::cmd(),
    ]
}

// exec given cmds (action, resource), client and args
pub async fn exec(
    action: &str,
    resource: &str,
    client: ApiClient,
    args: &clap::ArgMatches,
) -> Result<()> {
    match (action, resource) {
        ("get", "manifest") => get::exec_manifest(client, args).await,
        ("get", "build") => get::exec_build(client, args).await,
        ("get", "app") => get::exec_app(client, args).await,
        ("get", "log") => get::exec_build_log(client, args).await,
        ("create", "manifest") => create::exec_manifest(client, args).await,
        ("create", "build") => create::exec_build(client, args).await,
        ("list", "build-snapshot") => list::exec_build_snapshot(client, args).await,
        ("list", "manifest-snapshot") => list::exec_manifest_snapshot(client, args).await,
        ("list", "build") => list::exec_build(client, args).await,
        ("list", "node") => list::exec_node(client, args).await,
        ("cancel", "build") => cancel::exec_build(client, args).await,
        ("scan", "builder") => scan::exec_builder(client, args).await,
        _ => unreachable!("unknown cmd ({}, {})", action, resource),
    }
}
