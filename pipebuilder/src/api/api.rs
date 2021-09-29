pub mod filters {
    use super::{handlers, models};
    use pipebuilder_common::{
        grpc::{
            manifest::manifest_client::ManifestClient, schedule::scheduler_client::SchedulerClient,
        },
        Register,
    };
    use tonic::transport::Channel;
    use warp::Filter;

    pub fn api(
        _manifest_client: ManifestClient<Channel>,
        scheduler_client: SchedulerClient<Channel>,
        _register: Register,
        _lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        v1_build(scheduler_client)
    }

    pub fn v1_build(
        scheduler_client: SchedulerClient<Channel>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build")
            .and(warp::post())
            .and(json_build_request())
            .and(with_scheduler_client(scheduler_client))
            .and_then(handlers::build)
    }

    fn with_scheduler_client(
        client: SchedulerClient<Channel>,
    ) -> impl Filter<Extract = (SchedulerClient<Channel>,), Error = std::convert::Infallible> + Clone
    {
        warp::any().map(move || client.clone())
    }

    fn with_manifest_client(
        client: ManifestClient<Channel>,
    ) -> impl Filter<Extract = (ManifestClient<Channel>,), Error = std::convert::Infallible> + Clone
    {
        warp::any().map(move || client.clone())
    }

    fn with_register(
        register: Register,
    ) -> impl Filter<Extract = (Register,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || register.clone())
    }

    fn json_build_request(
    ) -> impl Filter<Extract = (models::BuildRequest,), Error = warp::Rejection> + Clone {
        // When accepting a body, we want a JSON body and reject huge payloads
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }
}

mod handlers {
    use super::models::{self, Failure};
    use pipebuilder_common::grpc::{
        build::{builder_client::BuilderClient, BuildRequest},
        schedule::{scheduler_client::SchedulerClient, ScheduleRequest, ScheduleResponse},
    };
    use std::convert::Infallible;
    use tonic::transport::Channel;
    use tracing::info;
    use warp::http::{Response, StatusCode};

    pub async fn build(
        request: models::BuildRequest,
        client: SchedulerClient<Channel>,
    ) -> Result<impl warp::Reply, Infallible> {
        let response = match schedule(client).await {
            Ok(response) => response,
            Err(err) => {
                return Ok(failure(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Failure::new(format!("{:#?}", err)),
                ))
            }
        };
        let builder_info = match response.builder_info {
            Some(builder_info) => builder_info,
            None => {
                return Ok(failure(
                    StatusCode::SERVICE_UNAVAILABLE,
                    Failure::new(String::from("builder unavailable")),
                ))
            }
        };
        let builder_id = builder_info.id;
        let builder_address = builder_info.address;
        info!("scheduled builder ({}, {})", builder_id, builder_address);
        let builder_client = match builder_client(builder_address).await {
            Ok(builder_client) => builder_client,
            Err(err) => {
                return Ok(failure(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Failure::new(format!("{:#?}", err)),
                ))
            }
        };
        let response = match do_build(builder_client, request).await {
            Ok(response) => response,
            Err(err) => {
                return Ok(failure(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Failure::new(format!("{:#?}", err)),
                ))
            }
        };
        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(serde_json::to_string(&response).unwrap()))
    }

    async fn builder_client(address: String) -> pipebuilder_common::Result<BuilderClient<Channel>> {
        let client = BuilderClient::connect(address).await?;
        Ok(client)
    }

    async fn do_build(
        mut client: BuilderClient<Channel>,
        request: models::BuildRequest,
    ) -> pipebuilder_common::Result<models::BuildResponse> {
        let request: BuildRequest = request.into();
        let response = client.build(request).await?;
        Ok(response.into_inner().into())
    }

    async fn schedule(
        mut client: SchedulerClient<Channel>,
    ) -> pipebuilder_common::Result<ScheduleResponse> {
        let response = client.schedule(ScheduleRequest {}).await?;
        Ok(response.into_inner())
    }

    fn failure(status_code: StatusCode, failure: Failure) -> http::Result<Response<String>> {
        Response::builder()
            .status(status_code)
            .body(serde_json::to_string(&failure).unwrap())
    }
}

mod models {

    use pipebuilder_common::{grpc::build, BuildStatus};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct BuildRequest {
        pub namespace: String,
        pub manifest_id: String,
        pub manifest_version: u64,
        pub target_platform: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct BuildResponse {
        pub build_version: u64,
    }

    #[derive(Serialize, Deserialize)]
    pub struct GetBuildRequest {
        pub namespace: String,
        pub manifest_id: String,
        pub build_version: u64,
    }

    #[derive(Serialize, Deserialize)]
    pub struct GetBuildResponse {
        pub status: BuildStatus,
    }

    #[derive(Serialize, Deserialize)]
    pub struct PutManifestRequest {
        pub namespace: String,
        pub id: Option<String>,
        pub buffer: Vec<u8>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct PutManifestResponse {
        pub id: String,
        pub version: u64,
    }

    #[derive(Serialize, Deserialize)]
    pub struct Failure {
        pub error: String,
    }

    impl Failure {
        pub fn new(error: String) -> Self {
            Failure { error }
        }
    }

    impl From<BuildRequest> for build::BuildRequest {
        fn from(origin: BuildRequest) -> Self {
            let namespace = origin.namespace;
            let manifest_id = origin.manifest_id;
            let manifest_version = origin.manifest_version;
            let target_platform = origin.target_platform;
            build::BuildRequest {
                namespace,
                manifest_id,
                manifest_version,
                target_platform,
            }
        }
    }

    impl From<build::BuildResponse> for BuildResponse {
        fn from(origin: build::BuildResponse) -> Self {
            let build_version = origin.version;
            BuildResponse { build_version }
        }
    }
}
