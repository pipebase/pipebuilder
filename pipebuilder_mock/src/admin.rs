pub mod filters {
    use super::handlers;
    use crate::utils;
    use pipebuilder_common::api::models;
    use tokio::sync::mpsc;
    use warp::Filter;

    pub fn admin(
        tx: mpsc::Sender<()>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        admin_shutdown(tx)
    }

    pub fn admin_shutdown(
        tx: mpsc::Sender<()>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("admin" / "shutdown")
            .and(warp::post())
            .and(utils::filters::with_shutdown_tx(tx))
            .and(utils::filters::json_request::<models::ShutdownRequest>())
            .and_then(handlers::shutdown)
    }
}

mod handlers {
    use crate::utils;
    use pipebuilder_common::api::models;
    use std::convert::Infallible;
    use tokio::sync::mpsc;

    async fn do_shutdown(
        tx: &mpsc::Sender<()>,
        _request: models::ShutdownRequest,
    ) -> pipebuilder_common::Result<models::ShutdownResponse> {
        // TODO: pipebuilder_common::Error handle mpsc::SendError
        let _ = tx.send(()).await;
        Ok(models::ShutdownResponse {})
    }

    pub async fn shutdown(
        tx: mpsc::Sender<()>,
        request: models::ShutdownRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_shutdown(&tx, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }
}
