pub mod filters {

    use super::handlers;
    use crate::utils;
    use pipebuilder_common::{api::models, Register};
    use warp::Filter;

    // node api
    pub fn v1_node(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        v1_node_state_list(register.clone())
            .or(v1_node_activate(register.clone(), lease_id))
            .or(v1_node_deactivate(register.clone(), lease_id))
            .or(v1_node_shutdown(register, lease_id))
    }

    pub fn v1_node_state_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "node")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::ListNodeStateRequest>())
            .and_then(handlers::list_node_state)
    }

    pub fn v1_node_activate(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "node" / "activate")
            .and(warp::post())
            .and(utils::filters::with_register(register))
            .and(utils::filters::with_lease_id(lease_id))
            .and(utils::filters::json_request::<models::ActivateNodeRequest>())
            .and_then(handlers::activate_node)
    }

    pub fn v1_node_deactivate(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "node" / "deactivate")
            .and(warp::post())
            .and(utils::filters::with_register(register))
            .and(utils::filters::with_lease_id(lease_id))
            .and(utils::filters::json_request::<models::DeactivateNodeRequest>())
            .and_then(handlers::deactivate_node)
    }

    pub fn v1_node_shutdown(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "node" / "shutdown")
            .and(warp::post())
            .and(utils::filters::with_register(register))
            .and(utils::filters::with_lease_id(lease_id))
            .and(utils::filters::json_request::<models::ShutdownNodeRequest>())
            .and_then(handlers::shutdown_node)
    }
}

mod handlers {

    use crate::{utils, validations};
    use pipebuilder_common::{
        api::models::{self, Failure},
        grpc::node::{
            node_client::NodeClient, ActivateRequest, DeactivateRequest, ShutdownRequest,
        },
        remove_resource, NodeState, Register,
    };
    use std::convert::Infallible;
    use tonic::transport::Channel;

    pub async fn list_node_state(
        mut register: Register,
        request: models::ListNodeStateRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate list node state request
        match validations::validate_list_node_state_request(&request) {
            Ok(()) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_list_node_state(&mut register, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_list_node_state(
        register: &mut Register,
        request: models::ListNodeStateRequest,
    ) -> pipebuilder_common::Result<Vec<models::NodeState>> {
        let role = request.role.as_ref();
        let node_states = register.list_resource::<NodeState>(None, None).await?;
        let node_states = node_states
            .into_iter()
            .filter_map(|(key, node_state)| {
                let id = remove_resource::<NodeState>(key.as_str());
                let node_state = models::NodeState {
                    id: id.to_owned(),
                    role: node_state.role,
                    arch: node_state.arch,
                    os: node_state.os,
                    status: node_state.status,
                    timestamp: node_state.timestamp,
                };
                let role = match role {
                    Some(role) => role,
                    None => return Some(node_state),
                };
                if &node_state.role == role {
                    return Some(node_state);
                }
                None
            })
            .collect::<Vec<models::NodeState>>();
        Ok(node_states)
    }

    pub async fn activate_node(
        mut register: Register,
        lease_id: i64,
        request: models::ActivateNodeRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        let node_id = request.id.as_str();
        let node_state = match utils::handlers::get_internal_node_state(
            &mut register,
            lease_id,
            node_id,
        )
        .await
        {
            Ok(node_state) => node_state,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        // find node address
        let address = match node_state {
            Some(node_state) => node_state.external_address,
            None => {
                return Ok(utils::handlers::http_not_found(Failure::new(format!(
                    "node '(id = {})' not found",
                    node_id
                ))))
            }
        };
        let mut client = match utils::handlers::node_client(address.as_str()).await {
            Ok(node_client) => node_client,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        match do_activate_node(&mut client).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    pub async fn deactivate_node(
        mut register: Register,
        lease_id: i64,
        request: models::DeactivateNodeRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        let node_id = request.id.as_str();
        let node_state = match utils::handlers::get_internal_node_state(
            &mut register,
            lease_id,
            node_id,
        )
        .await
        {
            Ok(node_state) => node_state,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        // find node address
        let address = match node_state {
            Some(node_state) => node_state.external_address,
            None => {
                return Ok(utils::handlers::http_not_found(Failure::new(format!(
                    "node '(id = {})' not found",
                    node_id
                ))))
            }
        };
        let mut client = match utils::handlers::node_client(address.as_str()).await {
            Ok(node_client) => node_client,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        match do_deactivate_node(&mut client).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    pub async fn shutdown_node(
        mut register: Register,
        lease_id: i64,
        request: models::ShutdownNodeRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        let node_id = request.id.as_str();
        let node_state = match utils::handlers::get_internal_node_state(
            &mut register,
            lease_id,
            node_id,
        )
        .await
        {
            Ok(node_state) => node_state,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        // find node address
        let address = match node_state {
            Some(node_state) => node_state.external_address,
            None => {
                return Ok(utils::handlers::http_not_found(Failure::new(format!(
                    "node '(id = {})' not found",
                    node_id
                ))))
            }
        };
        let mut client = match utils::handlers::node_client(address.as_str()).await {
            Ok(node_client) => node_client,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        match do_shutdown_node(&mut client).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_activate_node(
        client: &mut NodeClient<Channel>,
    ) -> pipebuilder_common::Result<models::ActivateNodeResponse> {
        let response = client.activate(ActivateRequest {}).await?;
        let response = response.into_inner();
        Ok(response.into())
    }

    async fn do_deactivate_node(
        client: &mut NodeClient<Channel>,
    ) -> pipebuilder_common::Result<models::DeactivateNodeResponse> {
        let response = client.deactivate(DeactivateRequest {}).await?;
        let response = response.into_inner();
        Ok(response.into())
    }

    async fn do_shutdown_node(
        client: &mut NodeClient<Channel>,
    ) -> pipebuilder_common::Result<models::ShutdownNodeResponse> {
        let response = client.shutdown(ShutdownRequest {}).await?;
        let response = response.into_inner();
        Ok(response.into())
    }
}
