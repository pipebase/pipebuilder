pub mod filters {
    use super::{handlers, models};
    use pipebuilder_common::{
        grpc::{
            manifest::manifest_client::ManifestClient, schedule::scheduler_client::SchedulerClient,
        },
        Register,
    };
    use serde::de::DeserializeOwned;
    use tonic::transport::Channel;
    use warp::Filter;

    pub fn api(
        manifest_client: ManifestClient<Channel>,
        scheduler_client: SchedulerClient<Channel>,
        register: Register,
        _lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        v1_build(scheduler_client)
            .or(v1_manifest_put(manifest_client.to_owned()))
            .or(v1_manifest_get(manifest_client))
            .or(v1_manifest_snapshot_list(register.to_owned()))
            .or(v1_build_snapshot_list(register))
    }

    pub fn v1_build(
        scheduler_client: SchedulerClient<Channel>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build")
            .and(warp::post())
            .and(with_scheduler_client(scheduler_client))
            .and(json_request::<models::BuildRequest>())
            .and_then(handlers::build)
    }

    pub fn v1_manifest_put(
        manifest_client: ManifestClient<Channel>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "manifest")
            .and(warp::put())
            .and(with_manifest_client(manifest_client))
            .and(json_request::<models::PutManifestRequest>())
            .and_then(handlers::put_manifest)
    }

    pub fn v1_manifest_get(
        manifest_client: ManifestClient<Channel>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "manifest")
            .and(warp::get())
            .and(with_manifest_client(manifest_client))
            .and(warp::query::<models::GetManifestRequest>())
            .and_then(handlers::get_manifest)
    }

    pub fn v1_manifest_snapshot_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "manifest-snapshot")
            .and(warp::get())
            .and(with_register(register))
            .and(warp::query::<models::ListManifestSnapshotRequest>())
            .and_then(handlers::list_manifest_snapshot)
    }

    pub fn v1_build_snapshot_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build-snapshot")
            .and(warp::get())
            .and(with_register(register))
            .and(warp::query::<models::ListBuildSnapshotRequest>())
            .and_then(handlers::list_build_snapshot)
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

    fn json_request<T>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone
    where
        T: Send + DeserializeOwned,
    {
        // When accepting a body, we want a JSON body and reject huge payloads
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }
}

mod handlers {
    use super::models::{self, Failure};
    use pipebuilder_common::{
        grpc::{
            build::{builder_client::BuilderClient, BuildRequest},
            manifest::{manifest_client::ManifestClient, GetManifestRequest, PutManifestRequest},
            schedule::{scheduler_client::SchedulerClient, ScheduleRequest, ScheduleResponse},
        },
        remove_resource_namespace, Register, REGISTER_KEY_PREFIX_BUILD_SNAPSHOT,
        REGISTER_KEY_PREFIX_MANIFEST_SNAPSHOT,
    };
    use serde::Serialize;
    use std::convert::Infallible;
    use tonic::transport::Channel;
    use tracing::info;
    use warp::http::{Response, StatusCode};

    pub async fn build(
        client: SchedulerClient<Channel>,
        request: models::BuildRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        let response = match schedule(client).await {
            Ok(response) => response,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        let builder_info = match response.builder_info {
            Some(builder_info) => builder_info,
            None => {
                return Ok(http_service_unavailable(Failure::new(String::from(
                    "builder unavailable",
                ))))
            }
        };
        let builder_id = builder_info.id;
        let builder_address = builder_info.address;
        info!("scheduled builder ({}, {})", builder_id, builder_address);
        let builder_client = match builder_client(builder_address).await {
            Ok(builder_client) => builder_client,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        match do_build(builder_client, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    pub async fn put_manifest(
        client: ManifestClient<Channel>,
        request: models::PutManifestRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_put_manifest(client, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    pub async fn get_manifest(
        client: ManifestClient<Channel>,
        request: models::GetManifestRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_get_manifest(client, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    pub async fn list_manifest_snapshot(
        register: Register,
        request: models::ListManifestSnapshotRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_list_manifest_snapshot(register, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    pub async fn list_build_snapshot(
        register: Register,
        request: models::ListBuildSnapshotRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_list_build_snapshot(register, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
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

    async fn do_put_manifest(
        mut client: ManifestClient<Channel>,
        request: models::PutManifestRequest,
    ) -> pipebuilder_common::Result<models::PutManifestResponse> {
        let request: PutManifestRequest = request.into();
        let response = client.put_manifest(request).await?;
        Ok(response.into_inner().into())
    }

    async fn do_get_manifest(
        mut client: ManifestClient<Channel>,
        request: models::GetManifestRequest,
    ) -> pipebuilder_common::Result<models::GetManifestResponse> {
        let request: GetManifestRequest = request.into();
        let response = client.get_manifest(request).await?;
        Ok(response.into_inner().into())
    }

    async fn do_list_manifest_snapshot(
        mut register: Register,
        request: models::ListManifestSnapshotRequest,
    ) -> pipebuilder_common::Result<Vec<models::ManifestSnapshot>> {
        let namespace = request.namespace;
        let manifest_snapshots = register.list_manifest_snapshot(namespace.as_str()).await?;
        let snapshots: Vec<models::ManifestSnapshot> = manifest_snapshots
            .into_iter()
            .map(|(key, manifest_snapshot)| models::ManifestSnapshot {
                id: remove_resource_namespace(
                    key.as_str(),
                    REGISTER_KEY_PREFIX_MANIFEST_SNAPSHOT,
                    namespace.as_str(),
                )
                .to_owned(),
                latest_version: manifest_snapshot.latest_version,
            })
            .collect();
        Ok(snapshots)
    }

    async fn do_list_build_snapshot(
        mut register: Register,
        request: models::ListBuildSnapshotRequest,
    ) -> pipebuilder_common::Result<Vec<models::BuildSnapshot>> {
        let namespace = request.namespace;
        let build_snapshots = register.list_build_snapshot(namespace.as_str()).await?;
        let snapshots: Vec<models::BuildSnapshot> = build_snapshots
            .into_iter()
            .map(|(key, build_snapshot)| models::BuildSnapshot {
                id: remove_resource_namespace(
                    key.as_str(),
                    REGISTER_KEY_PREFIX_BUILD_SNAPSHOT,
                    namespace.as_str(),
                )
                .to_owned(),
                latest_version: build_snapshot.latest_version,
            })
            .collect();
        Ok(snapshots)
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

    fn http_internal_error(f: Failure) -> http::Result<Response<String>> {
        failure(StatusCode::INTERNAL_SERVER_ERROR, f)
    }

    fn http_service_unavailable(f: Failure) -> http::Result<Response<String>> {
        failure(StatusCode::SERVICE_UNAVAILABLE, f)
    }

    fn ok<T>(t: &T) -> http::Result<Response<String>>
    where
        T: ?Sized + Serialize,
    {
        Response::builder()
            .status(StatusCode::OK)
            .body(serde_json::to_string::<T>(t).unwrap())
    }

    impl From<pipebuilder_common::Error> for Failure {
        fn from(error: pipebuilder_common::Error) -> Self {
            Failure::new(format!("{:#?}", error))
        }
    }
}

mod models {

    use pipebuilder_common::{
        grpc::{build, manifest},
        BuildStatus,
    };
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
    pub struct GetManifestRequest {
        pub namespace: String,
        pub id: String,
        pub version: u64,
    }

    #[derive(Serialize, Deserialize)]
    pub struct GetManifestResponse {
        pub buffer: Vec<u8>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ListManifestSnapshotRequest {
        pub namespace: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ManifestSnapshot {
        pub id: String,
        pub latest_version: u64,
    }

    #[derive(Serialize, Deserialize)]
    pub struct ListBuildSnapshotRequest {
        pub namespace: String,
    }

    #[derive(Serialize, Deserialize)]
    pub struct BuildSnapshot {
        pub id: String,
        pub latest_version: u64,
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

    impl From<PutManifestRequest> for manifest::PutManifestRequest {
        fn from(origin: PutManifestRequest) -> Self {
            let namespace = origin.namespace;
            let id = origin.id;
            let buffer = origin.buffer;
            manifest::PutManifestRequest {
                namespace,
                id,
                buffer,
            }
        }
    }

    impl From<manifest::PutManifestResponse> for PutManifestResponse {
        fn from(origin: manifest::PutManifestResponse) -> Self {
            let id = origin.id;
            let version = origin.version;
            PutManifestResponse { id, version }
        }
    }

    impl From<GetManifestRequest> for manifest::GetManifestRequest {
        fn from(origin: GetManifestRequest) -> Self {
            let namespace = origin.namespace;
            let id = origin.id;
            let version = origin.version;
            manifest::GetManifestRequest {
                namespace,
                id,
                version,
            }
        }
    }

    impl From<manifest::GetManifestResponse> for GetManifestResponse {
        fn from(origin: manifest::GetManifestResponse) -> Self {
            let buffer = origin.buffer;
            GetManifestResponse { buffer }
        }
    }
}
