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
            .or(v1_manifest_put(repository_client.clone()))
            .or(v1_manifest_get(repository_client.clone()))
            .or(v1_manifest_snapshot_list(register.clone()))
            .or(v1_build_snapshot_list(register.clone()))
            .or(v1_build_get(register.clone(), lease_id))
            .or(v1_build_list(register.clone()))
            .or(v1_build_cancel(register.clone(), lease_id))
            .or(v1_app_get(repository_client.clone()))
            .or(v1_build_log_get(register.clone(), lease_id))
            .or(v1_node_state_list(register.clone()))
            .or(v1_builder_scan(register.clone(), lease_id))
            .or(v1_node_activate(register.clone(), lease_id))
            .or(v1_node_deactivate(register.clone(), lease_id))
            .or(v1_app_metadata_list(register.clone()))
            .or(v1_manifest_metadata_list(register.clone()))
            .or(v1_namespace_put(register.clone(), lease_id))
            .or(v1_project_put(register.clone(), lease_id))
            .or(v1_namespace_list(register.clone()))
            .or(v1_project_list(register.clone()))
            .or(v1_build_delete(register, lease_id))
            .or(v1_app_delete(repository_client.clone()))
            .or(v1_manifest_delete(repository_client))
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

    pub fn v1_build_log_get(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build" / "log")
            .and(warp::get())
            .and(with_register(register))
            .and(with_lease_id(lease_id))
            .and(warp::query::<models::GetBuildLogRequest>())
            .and_then(handlers::get_build_log)
    }

    pub fn v1_node_state_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "node")
            .and(warp::get())
            .and(with_register(register))
            .and(warp::query::<models::ListNodeStateRequest>())
            .and_then(handlers::list_node_state)
    }

    pub fn v1_builder_scan(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "builder" / "scan")
            .and(warp::get())
            .and(with_register(register))
            .and(with_lease_id(lease_id))
            .and(warp::query::<models::ScanBuilderRequest>())
            .and_then(handlers::scan_builder)
    }

    pub fn v1_node_activate(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "node" / "activate")
            .and(warp::post())
            .and(with_register(register))
            .and(with_lease_id(lease_id))
            .and(json_request::<models::ActivateNodeRequest>())
            .and_then(handlers::activate_node)
    }

    pub fn v1_node_deactivate(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "node" / "deactivate")
            .and(warp::post())
            .and(with_register(register))
            .and(with_lease_id(lease_id))
            .and(json_request::<models::DeactivateNodeRequest>())
            .and_then(handlers::deactivate_node)
    }

    pub fn v1_app_metadata_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "app" / "metadata")
            .and(warp::get())
            .and(with_register(register))
            .and(warp::query::<models::ListAppMetadataRequest>())
            .and_then(handlers::list_app_metadata)
    }

    pub fn v1_manifest_metadata_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "manifest" / "metadata")
            .and(warp::get())
            .and(with_register(register))
            .and(warp::query::<models::ListManifestMetadataRequest>())
            .and_then(handlers::list_manifest_metadata)
    }

    pub fn v1_namespace_put(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "namespace")
            .and(warp::post())
            .and(with_register(register))
            .and(with_lease_id(lease_id))
            .and(json_request::<models::UpdateNamespaceRequest>())
            .and_then(handlers::put_namespace)
    }

    pub fn v1_project_put(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "project")
            .and(warp::post())
            .and(with_register(register))
            .and(with_lease_id(lease_id))
            .and(json_request::<models::UpdateProjectRequest>())
            .and_then(handlers::put_project)
    }

    pub fn v1_namespace_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "namespace")
            .and(warp::get())
            .and(with_register(register))
            .and(warp::query::<models::ListNamespaceRequest>())
            .and_then(handlers::list_namespace)
    }

    pub fn v1_project_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "project")
            .and(warp::get())
            .and(with_register(register))
            .and(warp::query::<models::ListProjectRequest>())
            .and_then(handlers::list_project)
    }

    pub fn v1_build_delete(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build")
            .and(warp::delete())
            .and(with_register(register))
            .and(with_lease_id(lease_id))
            .and(json_request::<models::DeleteBuildRequest>())
            .and_then(handlers::delete_build)
    }

    pub fn v1_app_delete(
        repository_client: RepositoryClient<Channel>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "app")
            .and(warp::delete())
            .and(with_repository_client(repository_client))
            .and(json_request::<models::DeleteAppRequest>())
            .and_then(handlers::delete_app)
    }

    pub fn v1_manifest_delete(
        repository_client: RepositoryClient<Channel>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "manifest")
            .and(warp::delete())
            .and(with_repository_client(repository_client))
            .and(json_request::<models::DeleteManifestRequest>())
            .and_then(handlers::delete_manifest)
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
    use super::validations;
    use pipebuilder_common::{
        api::models::{self, Failure},
        build_builder_client, build_node_client,
        grpc::{
            build::{
                builder_client::BuilderClient, BuildRequest, CancelRequest, GetLogRequest,
                ScanRequest,
            },
            node::{node_client::NodeClient, ActivateRequest, DeactivateRequest, StatusRequest},
            repository::{
                repository_client::RepositoryClient, DeleteAppRequest, DeleteManifestRequest,
                GetAppRequest, GetManifestRequest, PutManifestRequest,
            },
            schedule::{scheduler_client::SchedulerClient, ScheduleRequest, ScheduleResponse},
        },
        node_role_prefix, remove_resource, remove_resource_namespace, NodeRole,
        NodeState as InternalNodeState, Register, RESOURCE_APP_METADATA, RESOURCE_BUILD_SNAPSHOT,
        RESOURCE_MANIFEST_METADATA, RESOURCE_MANIFEST_SNAPSHOT, RESOURCE_NAMESPACE, RESOURCE_NODE,
        RESOURCE_PROJECT, RESOURCE_VERSION_BUILD,
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
        // validate build request
        match validations::validate_build_request(&request) {
            Ok(_) => (),
            Err(err) => return Ok(http_bad_request(err.into())),
        };
        let namespace = request.namespace.clone();
        let id = request.id.clone();
        // find a builder
        let response = match schedule(&mut client, namespace, id).await {
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
        // check whether builder is active
        let mut node_client = match node_client(builder_address.as_str()).await {
            Ok(node_client) => node_client,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        let active = match is_node_status_active(&mut node_client).await {
            Ok(active) => active,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        if !active {
            return Ok(http_service_unavailable(Failure::new(format!(
                "builder '{}' is inactive",
                builder_id
            ))));
        }
        // trigger the build
        let mut builder_client = match builder_client(builder_address.as_str()).await {
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

    async fn is_node_status_active(
        client: &mut NodeClient<Channel>,
    ) -> pipebuilder_common::Result<bool> {
        let response = client.status(StatusRequest {}).await?;
        let active = response.into_inner().active;
        Ok(active)
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
                    RESOURCE_MANIFEST_SNAPSHOT,
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
                    RESOURCE_BUILD_SNAPSHOT,
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
        let version_builds = register.list_version_build(namespace.as_str(), id).await?;
        let version_builds = version_builds
            .into_iter()
            .map(|(key, version_build)| {
                let id_version = remove_resource_namespace(
                    key.as_str(),
                    RESOURCE_VERSION_BUILD,
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
        let request_clone = request.clone();
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
        info!(
            "cancel build at builder ({}, {})",
            builder_id, builder_address
        );
        let mut builder_client = match builder_client(builder_address.as_str()).await {
            Ok(builder_client) => builder_client,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        match do_cancel_build(&mut builder_client, request_clone).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_cancel_build(
        client: &mut BuilderClient<Channel>,
        request: models::CancelBuildRequest,
    ) -> pipebuilder_common::Result<models::CancelBuildResponse> {
        let request: CancelRequest = request.into();
        let resp = client.cancel(request).await?;
        Ok(resp.into_inner().into())
    }

    pub async fn delete_build(
        mut register: Register,
        lease_id: i64,
        request: models::DeleteBuildRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        let namespace = request.namespace.as_str();
        let id = request.id.as_str();
        let version = request.version;
        let version_build = match register
            .get_version_build(lease_id, namespace, id, version)
            .await
        {
            Ok(version_build) => version_build,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        let version_build = match version_build {
            Some(version_build) => version_build,
            None => {
                return Ok(http_bad_request(Failure::new(format!(
                    "build {}/{}/{} not found",
                    namespace, id, version
                ))))
            }
        };
        if !version_build.is_stopped() {
            return Ok(http_bad_request(Failure::new(format!(
                "build {}/{}/{} is running, cancel required before delete",
                namespace, id, version
            ))));
        }
        match do_delete_build(&mut register, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_delete_build(
        register: &mut Register,
        request: models::DeleteBuildRequest,
    ) -> pipebuilder_common::Result<models::DeleteBuildResponse> {
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        register
            .delete_version_build(namespace.as_str(), id.as_str(), version)
            .await?;
        Ok(models::DeleteBuildResponse {})
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

    pub async fn delete_app(
        mut client: RepositoryClient<Channel>,
        request: models::DeleteAppRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_delete_app(&mut client, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_delete_app(
        client: &mut RepositoryClient<Channel>,
        request: models::DeleteAppRequest,
    ) -> pipebuilder_common::Result<models::DeleteAppResponse> {
        let request: DeleteAppRequest = request.into();
        let response = client.delete_app(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn delete_manifest(
        mut client: RepositoryClient<Channel>,
        request: models::DeleteManifestRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_delete_manifest(&mut client, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_delete_manifest(
        client: &mut RepositoryClient<Channel>,
        request: models::DeleteManifestRequest,
    ) -> pipebuilder_common::Result<models::DeleteManifestResponse> {
        let request: DeleteManifestRequest = request.into();
        let response = client.delete_manifest(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn get_build_log(
        mut register: Register,
        lease_id: i64,
        request: models::GetBuildLogRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        let request_clone = request.clone();
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
        // get local build log at builder
        let builder_id = version_build.builder_id;
        let builder_address = version_build.builder_address;
        info!(
            "get build log at builder ({}, {})",
            builder_id, builder_address
        );
        let mut builder_client = match builder_client(builder_address.as_str()).await {
            Ok(builder_client) => builder_client,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        match do_get_build_log(&mut builder_client, request_clone).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_get_build_log(
        client: &mut BuilderClient<Channel>,
        request: models::GetBuildLogRequest,
    ) -> pipebuilder_common::Result<models::GetBuildLogResponse> {
        let request: GetLogRequest = request.into();
        let resp = client.get_log(request).await?;
        Ok(resp.into_inner().into())
    }

    pub async fn list_node_state(
        mut register: Register,
        request: models::ListNodeStateRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate list node state request
        match validations::validate_list_node_state_request(&request) {
            Ok(()) => (),
            Err(err) => return Ok(http_bad_request(err.into())),
        };
        match do_list_node_state(&mut register, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_list_node_state(
        register: &mut Register,
        request: models::ListNodeStateRequest,
    ) -> pipebuilder_common::Result<Vec<models::NodeState>> {
        let role = request.role;
        let prefix = match role {
            Some(role) => node_role_prefix(role),
            None => RESOURCE_NODE,
        };
        let node_states = register.list_node_state(prefix).await?;
        let node_states = node_states
            .into_iter()
            .map(|(_, node_state)| node_state.into())
            .collect::<Vec<models::NodeState>>();
        Ok(node_states)
    }

    pub async fn scan_builder(
        mut register: Register,
        lease_id: i64,
        request: models::ScanBuilderRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        let builder_id = request.id.as_str();
        let node_state =
            match get_internal_node_state(&mut register, lease_id, NodeRole::Builder, builder_id)
                .await
            {
                Ok(node_state) => node_state,
                Err(err) => return Ok(http_internal_error(err.into())),
            };
        // find builder address
        let address = match node_state {
            Some(node_state) => node_state.external_address,
            None => {
                return Ok(http_not_found(Failure::new(format!(
                    "builder '{}' not found",
                    builder_id
                ))))
            }
        };
        let mut client = match builder_client(address.as_str()).await {
            Ok(client) => client,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        match do_scan_builder(&mut client, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_scan_builder(
        client: &mut BuilderClient<Channel>,
        request: models::ScanBuilderRequest,
    ) -> pipebuilder_common::Result<Vec<models::VersionBuildKey>> {
        let request: ScanRequest = request.into();
        let response = client.scan(request).await?;
        let response = response.into_inner();
        let builds = response.builds;
        let builds = builds
            .into_iter()
            .map(|b| b.into())
            .collect::<Vec<models::VersionBuildKey>>();
        Ok(builds)
    }

    pub async fn activate_node(
        mut register: Register,
        lease_id: i64,
        request: models::ActivateNodeRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match validations::validate_activate_node_request(&request) {
            Ok(()) => (),
            Err(err) => return Ok(http_bad_request(err.into())),
        };
        let node_id = request.id.as_str();
        let node_role = request.role;
        let node_state = match get_internal_node_state(
            &mut register,
            lease_id,
            node_role.clone(),
            node_id,
        )
        .await
        {
            Ok(node_state) => node_state,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        // find node address
        let address = match node_state {
            Some(node_state) => node_state.external_address,
            None => {
                return Ok(http_not_found(Failure::new(format!(
                    "node '({}, {})' not found",
                    node_role.to_string(),
                    node_id
                ))))
            }
        };
        let mut client = match node_client(address.as_str()).await {
            Ok(node_client) => node_client,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        match do_activate_node(&mut client).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    pub async fn deactivate_node(
        mut register: Register,
        lease_id: i64,
        request: models::DeactivateNodeRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match validations::validate_deactivate_node_request(&request) {
            Ok(()) => (),
            Err(err) => return Ok(http_bad_request(err.into())),
        };
        let node_id = request.id.as_str();
        let node_role = request.role;
        let node_state = match get_internal_node_state(
            &mut register,
            lease_id,
            node_role.clone(),
            node_id,
        )
        .await
        {
            Ok(node_state) => node_state,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        // find node address
        let address = match node_state {
            Some(node_state) => node_state.external_address,
            None => {
                return Ok(http_not_found(Failure::new(format!(
                    "node '({}, {})' not found",
                    node_role.to_string(),
                    node_id
                ))))
            }
        };
        let mut client = match node_client(address.as_str()).await {
            Ok(node_client) => node_client,
            Err(err) => return Ok(http_internal_error(err.into())),
        };
        match do_deactivate_node(&mut client).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
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

    pub async fn list_app_metadata(
        mut register: Register,
        request: models::ListAppMetadataRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_list_app_metadata(&mut register, request).await {
            Ok(resp) => Ok(ok(&resp)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_list_app_metadata(
        register: &mut Register,
        request: models::ListAppMetadataRequest,
    ) -> pipebuilder_common::Result<Vec<models::AppMetadata>> {
        let namespace = request.namespace.as_str();
        let id = request.id;
        let metas = register.list_app_metadata(namespace, id).await?;
        let metas = metas
            .into_iter()
            .map(|(key, meta)| {
                let id_version =
                    remove_resource_namespace(key.as_str(), RESOURCE_APP_METADATA, namespace);
                let id_version = id_version.split('/').collect::<Vec<&str>>();
                let id = id_version.get(0).expect("id not found in key").to_string();
                let version: u64 = id_version
                    .get(1)
                    .expect("version not found in key")
                    .parse()
                    .expect("cannot parse version as u64");
                models::AppMetadata {
                    id,
                    version,
                    pulls: meta.pulls,
                    size: meta.size,
                    created: meta.created,
                }
            })
            .collect::<Vec<models::AppMetadata>>();
        Ok(metas)
    }

    pub async fn list_manifest_metadata(
        mut register: Register,
        request: models::ListManifestMetadataRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_list_manifest_metadata(&mut register, request).await {
            Ok(resp) => Ok(ok(&resp)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_list_manifest_metadata(
        register: &mut Register,
        request: models::ListManifestMetadataRequest,
    ) -> pipebuilder_common::Result<Vec<models::ManifestMetadata>> {
        let namespace = request.namespace.as_str();
        let id = request.id;
        let metas = register.list_manifest_metadata(namespace, id).await?;
        let metas = metas
            .into_iter()
            .map(|(key, meta)| {
                let id_version =
                    remove_resource_namespace(key.as_str(), RESOURCE_MANIFEST_METADATA, namespace);
                let id_version = id_version.split('/').collect::<Vec<&str>>();
                let id = id_version.get(0).expect("id not found in key").to_string();
                let version: u64 = id_version
                    .get(1)
                    .expect("version not found in key")
                    .parse()
                    .expect("cannot parse version as u64");
                models::ManifestMetadata {
                    id,
                    version,
                    pulls: meta.pulls,
                    size: meta.size,
                    created: meta.created,
                }
            })
            .collect::<Vec<models::ManifestMetadata>>();
        Ok(metas)
    }

    async fn do_put_namespace(
        register: &mut Register,
        lease_id: i64,
        request: models::UpdateNamespaceRequest,
    ) -> pipebuilder_common::Result<models::Namespace> {
        let id = request.id;
        let (_, namespace) = register.update_namespace(lease_id, id.as_str()).await?;
        let created = namespace.created;
        Ok(models::Namespace { id, created })
    }

    pub async fn put_namespace(
        mut register: Register,
        lease_id: i64,
        request: models::UpdateNamespaceRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_put_namespace(&mut register, lease_id, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_put_project(
        register: &mut Register,
        lease_id: i64,
        request: models::UpdateProjectRequest,
    ) -> pipebuilder_common::Result<models::Project> {
        let namespace = request.namespace;
        let id = request.id;
        let (_, project) = register
            .update_project(lease_id, namespace.as_str(), id.as_str())
            .await?;
        let created = project.created;
        Ok(models::Project { id, created })
    }

    pub async fn put_project(
        mut register: Register,
        lease_id: i64,
        request: models::UpdateProjectRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_put_project(&mut register, lease_id, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_list_namespace(
        register: &mut Register,
        _request: models::ListNamespaceRequest,
    ) -> pipebuilder_common::Result<Vec<models::Namespace>> {
        let namespaces = register.list_namespace().await?;
        let namespaces = namespaces
            .into_iter()
            .map(|(key, namespace)| {
                let id = remove_resource(key.as_str(), RESOURCE_NAMESPACE);
                models::Namespace {
                    id: id.to_owned(),
                    created: namespace.created,
                }
            })
            .collect::<Vec<models::Namespace>>();
        Ok(namespaces)
    }

    pub async fn list_namespace(
        mut register: Register,
        _request: models::ListNamespaceRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_list_namespace(&mut register, _request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn do_list_project(
        register: &mut Register,
        request: models::ListProjectRequest,
    ) -> pipebuilder_common::Result<Vec<models::Project>> {
        let namespace = request.namespace;
        let projects = register.list_project(namespace.as_str()).await?;
        let projects = projects
            .into_iter()
            .map(|(key, project)| {
                let id =
                    remove_resource_namespace(key.as_str(), RESOURCE_PROJECT, namespace.as_str());
                models::Project {
                    id: id.to_owned(),
                    created: project.created,
                }
            })
            .collect::<Vec<models::Project>>();
        Ok(projects)
    }

    pub async fn list_project(
        mut register: Register,
        request: models::ListProjectRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_list_project(&mut register, request).await {
            Ok(response) => Ok(ok(&response)),
            Err(err) => Ok(http_internal_error(err.into())),
        }
    }

    async fn get_internal_node_state(
        register: &mut Register,
        lease_id: i64,
        role: NodeRole,
        id: &str,
    ) -> pipebuilder_common::Result<Option<InternalNodeState>> {
        register
            .get_node_state(lease_id, node_role_prefix(role), id)
            .await
    }

    async fn schedule(
        client: &mut SchedulerClient<Channel>,
        namespace: String,
        id: String,
    ) -> pipebuilder_common::Result<ScheduleResponse> {
        let response = client.schedule(ScheduleRequest { namespace, id }).await?;
        Ok(response.into_inner())
    }

    async fn builder_client(address: &str) -> pipebuilder_common::Result<BuilderClient<Channel>> {
        // TODO (Li Yu): configurable protocol
        build_builder_client("http", address).await
    }

    async fn node_client(address: &str) -> pipebuilder_common::Result<NodeClient<Channel>> {
        build_node_client("http", address).await
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

mod validations {

    use pipebuilder_common::{api::models, invalid_api_request, Build, NodeRole, Result};

    pub fn validate_build_request(request: &models::BuildRequest) -> Result<()> {
        if !Build::is_target_platform_support(request.target_platform.as_str()) {
            return Err(invalid_api_request(format!(
                "target platform '{}' not support",
                request.target_platform
            )));
        }
        Ok(())
    }

    pub fn validate_list_node_state_request(request: &models::ListNodeStateRequest) -> Result<()> {
        let role = request.role.as_ref();
        let role = match role {
            Some(role) => role,
            None => return Ok(()),
        };
        match role {
            NodeRole::Undefined => Err(invalid_api_request(String::from("undefined node role"))),
            _ => Ok(()),
        }
    }

    pub fn validate_activate_node_request(request: &models::ActivateNodeRequest) -> Result<()> {
        let role = &request.role;
        match role {
            NodeRole::Undefined => Err(invalid_api_request(String::from("undefined node role"))),
            _ => Ok(()),
        }
    }

    pub fn validate_deactivate_node_request(request: &models::DeactivateNodeRequest) -> Result<()> {
        let role = &request.role;
        match role {
            NodeRole::Undefined => Err(invalid_api_request(String::from("undefined node role"))),
            _ => Ok(()),
        }
    }
}
