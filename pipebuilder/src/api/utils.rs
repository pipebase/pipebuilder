pub mod filters {

    use pipebuilder_common::{
        grpc::{
            repository::repository_client::RepositoryClient,
            schedule::scheduler_client::SchedulerClient,
        },
        NodeService, Register,
    };
    use serde::de::DeserializeOwned;
    use tonic::transport::Channel;
    use warp::Filter;

    pub fn with_scheduler_client(
        client: SchedulerClient<Channel>,
    ) -> impl Filter<Extract = (SchedulerClient<Channel>,), Error = std::convert::Infallible> + Clone
    {
        warp::any().map(move || client.clone())
    }

    pub fn with_repository_client(
        client: RepositoryClient<Channel>,
    ) -> impl Filter<Extract = (RepositoryClient<Channel>,), Error = std::convert::Infallible> + Clone
    {
        warp::any().map(move || client.clone())
    }

    pub fn with_register(
        register: Register,
    ) -> impl Filter<Extract = (Register,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || register.clone())
    }

    pub fn with_lease_id(
        lease_id: i64,
    ) -> impl Filter<Extract = (i64,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || lease_id)
    }

    pub fn with_node_service(
        node_svc: NodeService,
    ) -> impl Filter<Extract = (NodeService,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || node_svc.clone())
    }

    pub fn json_request<T>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone
    where
        T: Send + DeserializeOwned,
    {
        // When accepting a body, we want a JSON body and reject huge payloads
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }
}

pub mod handlers {

    use pipebuilder_common::{
        api::models::Failure,
        grpc::{
            build::builder_client::BuilderClient,
            client::{BuilderClientBuilder, NodeClientBuilder, RpcProtocolType},
            node::{node_client::NodeClient, StatusRequest},
        },
        NodeState, Register,
    };
    use serde::Serialize;
    use tonic::transport::Channel;
    use warp::http::{Response, StatusCode};

    pub async fn get_internal_node_state(
        register: &mut Register,
        lease_id: i64,
        id: &str,
    ) -> pipebuilder_common::Result<Option<NodeState>> {
        register.get_resource(None, id, None, lease_id).await
    }

    pub async fn is_node_status_active(
        client: &mut NodeClient<Channel>,
    ) -> pipebuilder_common::Result<bool> {
        let response = client.status(StatusRequest {}).await?;
        let active = response.into_inner().active;
        Ok(active)
    }

    pub async fn builder_client(
        address: &str,
    ) -> pipebuilder_common::Result<BuilderClient<Channel>> {
        // TODO (Li Yu): configurable protocol
        BuilderClientBuilder::default()
            .protocol(RpcProtocolType::Http)
            .address(address)
            .connect()
            .await
    }

    pub async fn node_client(address: &str) -> pipebuilder_common::Result<NodeClient<Channel>> {
        NodeClientBuilder::default()
            .protocol(RpcProtocolType::Http)
            .address(address)
            .connect()
            .await
    }

    pub fn failure(status_code: StatusCode, failure: Failure) -> http::Result<Response<String>> {
        Response::builder()
            .status(status_code)
            .body(serde_json::to_string(&failure).unwrap())
    }

    pub fn http_internal_error(f: Failure) -> http::Result<Response<String>> {
        failure(StatusCode::INTERNAL_SERVER_ERROR, f)
    }

    pub fn http_service_unavailable(f: Failure) -> http::Result<Response<String>> {
        failure(StatusCode::SERVICE_UNAVAILABLE, f)
    }

    pub fn http_not_found(f: Failure) -> http::Result<Response<String>> {
        failure(StatusCode::NOT_FOUND, f)
    }

    pub fn http_bad_request(f: Failure) -> http::Result<Response<String>> {
        failure(StatusCode::BAD_REQUEST, f)
    }

    pub fn ok<T>(t: &T) -> http::Result<Response<String>>
    where
        T: ?Sized + Serialize,
    {
        Response::builder()
            .status(StatusCode::OK)
            .body(serde_json::to_string::<T>(t).unwrap())
    }
}
