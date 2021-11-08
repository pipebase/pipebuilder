use crate::{api, config::ApiConfig};
use pipebuilder_common::{
    grpc::{
        repository::repository_client::RepositoryClient,
        schedule::scheduler_client::SchedulerClient,
    },
    Register, Result,
};
use warp::Filter;

pub async fn bootstrap(
    config: ApiConfig,
    register: Register,
    lease_id: i64,
) -> Result<impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone> {
    let clients = config.clients;
    let repository_endpoint = clients.repository.endpoint;
    let scheduler_endpoint = clients.scheduler.endpoint;
    let manifest_client = RepositoryClient::connect(repository_endpoint).await?;
    let scheduler_client = SchedulerClient::connect(scheduler_endpoint).await?;
    let api = api::filters::api(manifest_client, scheduler_client, register, lease_id);
    Ok(api)
}
