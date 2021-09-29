use crate::{api, config::ApiConfig};
use pipebuilder_common::{
    grpc::{
        manifest::manifest_client::ManifestClient, schedule::scheduler_client::SchedulerClient,
    },
    Register, Result,
};
use warp::Filter;
// use tonic::transport::Channel;

pub async fn bootstrap(
    config: ApiConfig,
    register: Register,
    lease_id: i64,
) -> Result<impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone> {
    let clients = config.clients;
    let manifest_endpoint = clients.manifest.endpoint;
    let scheduler_endpoint = clients.scheduler.endpoint;
    let manifest_client = ManifestClient::connect(manifest_endpoint).await?;
    let scheduler_client = SchedulerClient::connect(scheduler_endpoint).await?;
    let api = api::filters::api(manifest_client, scheduler_client, register, lease_id);
    Ok(api)
}
