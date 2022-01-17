use crate::{api, config::MockRepository};
use pipebuilder_common::Result;
use std::path::PathBuf;
use tokio::sync::mpsc;
use warp::Filter;

pub fn bootstrap(
    repository: MockRepository,
) -> Result<(
    impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone,
    mpsc::Receiver<()>,
)> {
    let app_directory = PathBuf::from(repository.app);
    let catalogs_directory = PathBuf::from(repository.catalogs);
    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);
    let api = api::filters::api(app_directory, catalogs_directory, shutdown_tx);
    Ok((api, shutdown_rx))
}
