use pipebuilder_common::{api::client::ApiClient, Result};

pub(crate) mod activate;
pub(crate) mod cancel;
pub(crate) mod create;
pub(crate) mod deactivate;
pub(crate) mod delete;
pub(crate) mod get;
pub(crate) mod list;
pub(crate) mod pull;
pub(crate) mod push;
pub(crate) mod scan;
pub(crate) mod shutdown;

pub type Cmd = clap::App<'static>;

pub fn cmds() -> Vec<Cmd> {
    vec![
        activate::cmd(),
        create::cmd(),
        deactivate::cmd(),
        delete::cmd(),
        get::cmd(),
        list::cmd(),
        cancel::cmd(),
        scan::cmd(),
        pull::cmd(),
        push::cmd(),
        shutdown::cmd(),
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
        ("activate", "builder") => activate::exec_builder(client, args).await,
        ("deactivate", "builder") => deactivate::exec_builder(client, args).await,
        ("shutdown", "builder") => shutdown::exec_builder(client, args).await,
        ("shutdown", "repository") => shutdown::exec_repository(client, args).await,
        ("shutdown", "scheduler") => shutdown::exec_scheduler(client, args).await,
        ("get", "build") => get::exec_build(client, args).await,
        ("pull", "app") => pull::exec_app(client, args).await,
        ("pull", "manifest") => pull::exec_manifest(client, args).await,
        ("pull", "log") => pull::exec_build_log(client, args).await,
        ("push", "manifest") => push::exec_manifest(client, args).await,
        ("create", "build") => create::exec_build(client, args).await,
        ("create", "namespace") => create::exec_namespace(client, args).await,
        ("create", "project") => create::exec_project(client, args).await,
        ("list", "build") => list::exec_build(client, args).await,
        ("list", "manifest") => list::exec_manifest(client, args).await,
        ("list", "node") => list::exec_node(client, args).await,
        ("list", "app") => list::exec_app(client, args).await,
        ("list", "namespace") => list::exec_namespace(client, args).await,
        ("list", "project") => list::exec_project(client, args).await,
        ("cancel", "build") => cancel::exec_build(client, args).await,
        ("scan", "build") => scan::exec_build(client, args).await,
        ("scan", "build-cache") => scan::exec_build_cache(client, args).await,
        ("delete", "app") => delete::exec_app(client, args).await,
        ("delete", "build") => delete::exec_build(client, args).await,
        ("delete", "build-cache") => delete::exec_build_cache(client, args).await,
        ("delete", "manifest") => delete::exec_manifest(client, args).await,
        ("delete", "project") => delete::exec_project(client, args).await,
        ("delete", "namespace") => delete::exec_namespace(client, args).await,
        _ => unreachable!("unknown cmd ({}, {})", action, resource),
    }
}
