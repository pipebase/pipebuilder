pub mod filters {
    use super::handlers;
    use crate::utils;
    use pipebuilder_common::{api::models, NodeService};
    use warp::Filter;

    pub fn admin(
        node_svc: NodeService,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        admin_shutdown(node_svc)
    }

    pub fn admin_shutdown(
        node_svc: NodeService,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("admin" / "shutdown")
            .and(warp::post())
            .and(utils::filters::with_node_service(node_svc))
            .and(utils::filters::json_request::<models::ShutdownRequest>())
            .and_then(handlers::shutdown)
    }
}

mod handlers {
    use crate::utils;
    use pipebuilder_common::{
        api::models,
        grpc::node::{node_server::Node, ShutdownRequest},
        NodeService,
    };
    use std::convert::Infallible;
    use tonic::IntoRequest;

    async fn do_shutdown(
        node_svc: &NodeService,
        request: models::ShutdownRequest,
    ) -> pipebuilder_common::Result<models::ShutdownResponse> {
        let request: ShutdownRequest = request.into();
        let response = node_svc.shutdown(request.into_request()).await?;
        Ok(response.into_inner().into())
    }

    pub async fn shutdown(
        node_svc: NodeService,
        request: models::ShutdownRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_shutdown(&node_svc, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }
}
