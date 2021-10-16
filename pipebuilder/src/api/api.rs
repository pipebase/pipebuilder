pub mod filters {
    use super::handlers;
    use pipebuilder_common::{
        api::models,
        grpc::{
            repository::repository_client::RepositoryClient,
            schedule::scheduler_client::SchedulerClient,
        },
        Register,
    };
    use serde::de::DeserializeOwned;
    use tonic::transport::Channel;
    use warp::Filter;

    pub fn api(
        repository_client: RepositoryClient<Channel>,
        scheduler_client: SchedulerClient<Channel>,
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        v1_build(scheduler_client)
            .or(v1_manifest_put(repository_client.to_owned()))
            .or(v1_manifest_get(repository_client.to_owned()))
            .or(v1_manifest_snapshot_list(register.to_owned()))
            .or(v1_build_snapshot_list(register.to_owned()))
            .or(v1_build_get(register.to_owned(), lease_id))
            .or(v1_build_list(register.to_owned()))
            .or(v1_build_cancel(register, lease_id))
            .or(v1_app_get(repository_client))
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
        repository_client: RepositoryClient<Channel>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "manifest")
            .and(warp::post())
            .and(with_repository_client(repository_client))
            .and(json_request::<models::PutManifestRequest>())
            .and_then(handlers::put_manifest)
    }

    pub fn v1_manifest_get(
        repository_client: RepositoryClient<Channel>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "manifest")
            .and(warp::get())
            .and(with_repository_client(repository_client))
            .and(warp::query::<models::GetManifestRequest>())
            .and_then(handlers::get_manifest)
    }

    pub fn v1_manifest_snapshot_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "manifest" / "snapshot")
            .and(warp::get())
            .and(with_register(register))
            .and(warp::query::<models::ListManifestSnapshotRequest>())
            .and_then(handlers::list_manifest_snapshot)
    }

    pub fn v1_build_snapshot_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build" / "snapshot")
            .and(warp::get())
            .and(with_register(register))
            .and(warp::query::<models::ListBuildSnapshotRequest>())
            .and_then(handlers::list_build_snapshot)
    }

    pub fn v1_build_get(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build")
            .and(warp::get())
            .and(with_register(register))
            .and(with_lease_id(lease_id))
            .and(warp::query::<models::GetBuildRequest>())
            .and_then(handlers::get_build)
    }

    pub fn v1_build_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build")
            .and(warp::get())
            .and(with_register(register))
            .and(warp::query::<models::ListBuildRequest>())
            .and_then(handlers::list_build)
    }

    pub fn v1_build_cancel(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build" / "cancel")
            .and(warp::post())
            .and(with_register(register))
            .and(with_lease_id(lease_id))
            .and(json_request::<models::CancelBuildRequest>())
            .and_then(handlers::cancel_build)
    }

    pub fn v1_app_get(
        repository_client: RepositoryClient<Channel>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "app")
            .and(warp::get())
            .and(with_repository_client(repository_client))
            .and(warp::query::<models::GetAppRequest>())
            .and_then(handlers::get_app)
    }

    fn with_scheduler_client(
        client: SchedulerClient<Channel>,
    ) -> impl Filter<Extract = (SchedulerClient<Channel>,), Error = std::convert::Infallible> + Clone
    {
        warp::any().map(move || client.clone())
    }

    fn with_repository_client(
        client: RepositoryClient<Channel>,
    ) -> impl Filter<Extract = (RepositoryClient<Channel>,), Error = std::convert::Infallible> + Clone
    {
        warp::any().map(move || client.clone())
    }

    fn with_register(
        register: Register,
    ) -> impl Filter<Extract = (Register,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || register.clone())
    }

    fn with_lease_id(
        lease_id: i64,
    ) -> impl Filter<Extract = (i64,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || lease_id)
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
    use super::validate;
    use pipebuilder_common::{
        api::models::{self, Failure},
        grpc::{
            build::{builder_client::BuilderClient, BuildRequest, CancelRequest},
            repository::{
                repository_client::RepositoryClient, GetAppRequest, GetManifestRequest,
                PutManifestRequest,
            },
            schedule::{scheduler_client::SchedulerClient, ScheduleRequest, ScheduleResponse},
        },
        remove_resource_namespace, Register, REGISTER_KEY_PREFIX_BUILD_SNAPSHOT,
        REGISTER_KEY_PREFIX_MANIFEST_SNAPSHOT, REGISTER_KEY_PREFIX_VERSION_BUILD,
    };
    use serde::Serialize;
    use std::convert::Infallible;
    use tonic::transport::Channel;
    use tracing::info;
    use warp::http::{Response, StatusCode};

    pub async fn build(
        mut client: SchedulerClient<Channel>,
        request: models::BuildRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match validate::validate_build_request(&request) {
            Ok(_) => (),
            Err(err) => return Ok(http_bad_request(err.into())),
        };
        let response = match schedule(&mut client).await {
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
        let mut builder_client = match builder_client(builder_address).await {
            Ok(builder_client) => builder_client,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        match do_build(&mut builder_client, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_build(
        client: &mut BuilderClient<Channel>,
        request: models::BuildRequest,
    ) -> pipebuilder_common::Result<models::BuildResponse> {
        let request: BuildRequest = request.into();
        let response = client.build(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn put_manifest(
        mut client: RepositoryClient<Channel>,
        request: models::PutManifestRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_put_manifest(&mut client, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_put_manifest(
        client: &mut RepositoryClient<Channel>,
        request: models::PutManifestRequest,
    ) -> pipebuilder_common::Result<models::PutManifestResponse> {
        let request: PutManifestRequest = request.into();
        let response = client.put_manifest(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn get_manifest(
        mut client: RepositoryClient<Channel>,
        request: models::GetManifestRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_get_manifest(&mut client, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_not_found(err.into())),
        }
    }

    async fn do_get_manifest(
        client: &mut RepositoryClient<Channel>,
        request: models::GetManifestRequest,
    ) -> pipebuilder_common::Result<models::GetManifestResponse> {
        let request: GetManifestRequest = request.into();
        let response = client.get_manifest(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn list_manifest_snapshot(
        mut register: Register,
        request: models::ListManifestSnapshotRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_list_manifest_snapshot(&mut register, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_list_manifest_snapshot(
        register: &mut Register,
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

    pub async fn list_build_snapshot(
        mut register: Register,
        request: models::ListBuildSnapshotRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_list_build_snapshot(&mut register, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_list_build_snapshot(
        register: &mut Register,
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

    pub async fn get_build(
        mut register: Register,
        lease_id: i64,
        request: models::GetBuildRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        let response = match do_get_build(&mut register, lease_id, request).await {
            Ok(response) => response,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        match response {
            Some(response) => Ok(ok(&response)),
            None => Ok(http_not_found(Failure::new(String::from(
                "version build not found",
            )))),
        }
    }

    async fn do_get_build(
        register: &mut Register,
        lease_id: i64,
        request: models::GetBuildRequest,
    ) -> pipebuilder_common::Result<Option<models::VersionBuild>> {
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        let version_build = register
            .get_version_build(lease_id, namespace.as_str(), id.as_str(), version)
            .await?;
        Ok(version_build.map(|b| models::VersionBuild {
            id,
            version,
            status: b.status,
            timestamp: b.timestamp,
            builder_id: b.builder_id,
            builder_address: b.builder_address,
            message: b.message,
        }))
    }

    pub async fn list_build(
        mut register: Register,
        request: models::ListBuildRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_list_build(&mut register, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_list_build(
        register: &mut Register,
        request: models::ListBuildRequest,
    ) -> pipebuilder_common::Result<Vec<models::VersionBuild>> {
        let namespace = request.namespace;
        let id = request.id;
        let version_builds = register
            .list_version_build(namespace.as_str(), id.as_str())
            .await?;
        let version_builds = version_builds
            .into_iter()
            .map(|(key, version_build)| {
                let id_version = remove_resource_namespace(
                    key.as_str(),
                    REGISTER_KEY_PREFIX_VERSION_BUILD,
                    namespace.as_str(),
                );
                let id_version = id_version.split('/').collect::<Vec<&str>>();
                let id = id_version.get(0).expect("id not found in key").to_string();
                let version: u64 = id_version
                    .get(1)
                    .expect("version not found in key")
                    .parse()
                    .expect("cannot parse version as u64");
                models::VersionBuild {
                    id,
                    version,
                    status: version_build.status,
                    timestamp: version_build.timestamp,
                    builder_id: version_build.builder_id,
                    builder_address: version_build.builder_address,
                    message: version_build.message,
                }
            })
            .collect::<Vec<models::VersionBuild>>();
        Ok(version_builds)
    }

    pub async fn cancel_build(
        mut register: Register,
        lease_id: i64,
        request: models::CancelBuildRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // query version build for builder address
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        let version_build = match do_get_build(
            &mut register,
            lease_id,
            models::GetBuildRequest {
                namespace: namespace.clone(),
                id: id.clone(),
                version,
            },
        )
        .await
        {
            Ok(version_build) => version_build,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        let version_build = match version_build {
            Some(version_build) => version_build,
            None => {
                return Ok(http_not_found(Failure::new(format!(
                    "version build {}/{}/{} not found",
                    namespace, id, version
                ))))
            }
        };
        // cancel local build at builder
        let builder_id = version_build.builder_id;
        let builder_address = version_build.builder_address;
        info!("scheduled builder ({}, {})", builder_id, builder_address);
        let mut builder_client = match builder_client(builder_address).await {
            Ok(builder_client) => builder_client,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        match do_cancel_build(
            &mut builder_client,
            namespace.as_str(),
            id.as_str(),
            version,
        )
        .await
        {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_cancel_build(
        client: &mut BuilderClient<Channel>,
        namespace: &str,
        id: &str,
        version: u64,
    ) -> pipebuilder_common::Result<models::CancelBuildResponse> {
        let resp = client
            .cancel(CancelRequest {
                namespace: namespace.to_owned(),
                id: id.to_owned(),
                build_version: version,
            })
            .await?;
        Ok(resp.into_inner().into())
    }

    pub async fn get_app(
        mut client: RepositoryClient<Channel>,
        request: models::GetAppRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_get_app(&mut client, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_not_found(err.into())),
        }
    }

    async fn do_get_app(
        client: &mut RepositoryClient<Channel>,
        request: models::GetAppRequest,
    ) -> pipebuilder_common::Result<models::GetAppResponse> {
        let request: GetAppRequest = request.into();
        let response = client.get_app(request).await?;
        Ok(response.into_inner().into())
    }

    async fn schedule(
        client: &mut SchedulerClient<Channel>,
    ) -> pipebuilder_common::Result<ScheduleResponse> {
        let response = client.schedule(ScheduleRequest {}).await?;
        Ok(response.into_inner())
    }

    async fn builder_client(address: String) -> pipebuilder_common::Result<BuilderClient<Channel>> {
        // TODO (Li Yu): configurable protocol
        let client = BuilderClient::connect(format!("http://{}", address)).await?;
        Ok(client)
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

    fn http_not_found(f: Failure) -> http::Result<Response<String>> {
        failure(StatusCode::NOT_FOUND, f)
    }

    fn http_bad_request(f: Failure) -> http::Result<Response<String>> {
        failure(StatusCode::BAD_REQUEST, f)
    }

    fn ok<T>(t: &T) -> http::Result<Response<String>>
    where
        T: ?Sized + Serialize,
    {
        Response::builder()
            .status(StatusCode::OK)
            .body(serde_json::to_string::<T>(t).unwrap())
    }
}

mod validate {

    use pipebuilder_common::{api::models, invalid_api_request, Build, Result};

    pub fn validate_build_request(request: &models::BuildRequest) -> Result<()> {
        if !Build::is_target_platform_support(request.target_platform.as_str()) {
            return Err(invalid_api_request(format!(
                "target platform '{}' not support",
                request.target_platform
            )));
        }
        Ok(())
    }
}
