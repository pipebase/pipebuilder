use crate::{api, config::ApiConfig};
use pipebuilder_common::{
    grpc::client::{RepositoryClientBuilder, SchedulerClientBuilder},
    NodeService, Register, Result,
};
use tracing::info;
use warp::Filter;

pub async fn bootstrap(
    config: ApiConfig,
    register: Register,
    lease_id: i64,
    node_svc: NodeService,
) -> Result<impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone> {
    // connect internal sevices
    let clients = config.clients;
    // connect repository service
    let endpoint = clients.repository.endpoint();
    let protocol = clients.repository.protocol;
    let address = clients.repository.address;
    info!(
        endpoint = endpoint.as_str(),
        "connect repository service ..."
    );
    let repository_client = RepositoryClientBuilder::default()
        .protocol(protocol)
        .address(address.as_str())
        .connect()
        .await?;
    // connect scheduler service
    let endpoint = clients.scheduler.endpoint();
    let protocol = clients.scheduler.protocol;
    let address = clients.scheduler.address;
    info!(
        endpoint = endpoint.as_str(),
        "connect scheduler service ..."
    );
    let scheduler_client = SchedulerClientBuilder::default()
        .protocol(protocol)
        .address(address.as_str())
        .connect()
        .await?;
    let api = api::filters::api(
        repository_client,
        scheduler_client,
        register,
        lease_id,
        node_svc,
    );
    Ok(api)
}
