pub mod filters {
    use super::handlers;
    use crate::utils;
    use pipebuilder_common::{
        api::models, grpc::schedule::scheduler_client::SchedulerClient, Register,
    };
    use tonic::transport::Channel;
    use warp::Filter;

    // build api
    pub fn v1_build(
        scheduler_client: SchedulerClient<Channel>,
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        v1_build_post(scheduler_client, register.clone())
            .or(v1_build_snapshot_list(register.clone()))
            .or(v1_build_snapshot_delete(register.clone()))
            .or(v1_build_metadata_get(register.clone(), lease_id))
            .or(v1_build_metadata_list(register.clone()))
            .or(v1_build_cancel(register.clone(), lease_id))
            .or(v1_build_log_get(register.clone(), lease_id))
            .or(v1_build_delete(register.clone(), lease_id))
            .or(v1_build_scan(register.clone(), lease_id))
            .or(v1_build_cache_scan(register.clone(), lease_id))
            .or(v1_build_cache_delete(register, lease_id))
    }

    pub fn v1_build_post(
        scheduler_client: SchedulerClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build")
            .and(warp::post())
            .and(utils::filters::with_scheduler_client(scheduler_client))
            .and(utils::filters::with_register(register))
            .and(utils::filters::json_request::<models::BuildRequest>())
            .and_then(handlers::build)
    }

    pub fn v1_build_snapshot_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build" / "snapshot")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::ListBuildSnapshotRequest>())
            .and_then(handlers::list_build_snapshot)
    }

    pub fn v1_build_snapshot_delete(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build" / "snapshot")
            .and(warp::delete())
            .and(utils::filters::with_register(register))
            .and(utils::filters::json_request::<
                models::DeleteBuildSnapshotRequest,
            >())
            .and_then(handlers::delete_build_snapshot)
    }

    pub fn v1_build_metadata_get(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build" / "metadata")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(utils::filters::with_lease_id(lease_id))
            .and(warp::query::<models::GetBuildRequest>())
            .and_then(handlers::get_build_metadata)
    }

    pub fn v1_build_metadata_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build" / "metadata")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::ListBuildRequest>())
            .and_then(handlers::list_build_metadata)
    }

    pub fn v1_build_cancel(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build" / "cancel")
            .and(warp::post())
            .and(utils::filters::with_register(register))
            .and(utils::filters::with_lease_id(lease_id))
            .and(utils::filters::json_request::<models::CancelBuildRequest>())
            .and_then(handlers::cancel_build)
    }

    pub fn v1_build_delete(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build")
            .and(warp::delete())
            .and(utils::filters::with_register(register))
            .and(utils::filters::with_lease_id(lease_id))
            .and(utils::filters::json_request::<models::DeleteBuildRequest>())
            .and_then(handlers::delete_build_metadata)
    }

    pub fn v1_build_log_get(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build" / "log")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(utils::filters::with_lease_id(lease_id))
            .and(warp::query::<models::GetBuildLogRequest>())
            .and_then(handlers::get_build_log)
    }

    pub fn v1_build_scan(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build" / "scan")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(utils::filters::with_lease_id(lease_id))
            .and(warp::query::<models::ScanBuildRequest>())
            .and_then(handlers::scan_build)
    }

    pub fn v1_build_cache_scan(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build-cache" / "scan")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(utils::filters::with_lease_id(lease_id))
            .and(warp::query::<models::ScanBuildCacheRequest>())
            .and_then(handlers::scan_build_cache)
    }

    pub fn v1_build_cache_delete(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "build-cache")
            .and(warp::delete())
            .and(utils::filters::with_register(register))
            .and(utils::filters::with_lease_id(lease_id))
            .and(utils::filters::json_request::<
                models::DeleteBuildCacheRequest,
            >())
            .and_then(handlers::delete_build_cache)
    }
}

mod handlers {
    use crate::{utils, validations};
    use pipebuilder_common::{
        api::models::{self, Failure},
        grpc::{
            build::{
                builder_client::BuilderClient, BuildRequest, CancelBuildRequest,
                DeleteBuildCacheRequest, GetBuildLogRequest, ScanBuildCacheRequest,
                ScanBuildRequest,
            },
            schedule::{scheduler_client::SchedulerClient, ScheduleRequest, ScheduleResponse},
        },
        remove_resource_namespace, BuildMetadata, BuildSnapshot, NodeRole, Register,
    };
    use std::convert::Infallible;
    use tonic::transport::Channel;
    use tracing::info;

    pub async fn build(
        mut client: SchedulerClient<Channel>,
        mut register: Register,
        mut request: models::BuildRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate build request
        match validations::validate_build_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        let namespace = request.namespace.clone();
        let id = request.id.clone();
        let target_platform = request.target_platform.clone();
        // find a builder
        let response = match schedule(&mut client, namespace, id, target_platform.clone()).await {
            Ok(response) => response,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        let builder_info = match response.builder_info {
            Some(builder_info) => builder_info,
            None => {
                return Ok(utils::handlers::http_service_unavailable(Failure::new(
                    String::from("builder unavailable"),
                )))
            }
        };
        // target platform validation
        let builder_target_platform = builder_info.target_platform;
        match target_platform {
            Some(target_platform) => {
                if target_platform != builder_target_platform {
                    return Ok(utils::handlers::http_service_unavailable(Failure::new(
                        format!(
                            "builder target platform miss match '{}' != '{}'",
                            builder_target_platform, target_platform
                        ),
                    )));
                }
            }
            None => request.set_target_platform(builder_target_platform),
        };
        let builder_id = builder_info.id;
        let builder_address = builder_info.address;
        info!(
            builder_id = builder_id.as_str(),
            builder_address = builder_address.as_str(),
            "scheduled builder"
        );
        // check whether builder is active
        let mut node_client = match utils::handlers::node_client(builder_address.as_str()).await {
            Ok(node_client) => node_client,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        let active = match utils::handlers::is_node_status_active(&mut node_client).await {
            Ok(active) => active,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        if !active {
            return Ok(utils::handlers::http_service_unavailable(Failure::new(
                format!("builder '{}' is inactive", builder_id),
            )));
        }
        // trigger the build
        let mut builder_client =
            match utils::handlers::builder_client(builder_address.as_str()).await {
                Ok(builder_client) => builder_client,
                Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
            };
        match do_build(&mut builder_client, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn schedule(
        client: &mut SchedulerClient<Channel>,
        namespace: String,
        id: String,
        target_platform: Option<String>,
    ) -> pipebuilder_common::Result<ScheduleResponse> {
        let response = client
            .schedule(ScheduleRequest {
                namespace,
                id,
                target_platform,
            })
            .await?;
        Ok(response.into_inner())
    }

    async fn do_build(
        client: &mut BuilderClient<Channel>,
        request: models::BuildRequest,
    ) -> pipebuilder_common::Result<models::BuildResponse> {
        let request: BuildRequest = request.into();
        let response = client.build(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn list_build_snapshot(
        mut register: Register,
        request: models::ListBuildSnapshotRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_list_build_snapshot_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_list_build_snapshot(&mut register, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_list_build_snapshot(
        register: &mut Register,
        request: models::ListBuildSnapshotRequest,
    ) -> pipebuilder_common::Result<Vec<models::BuildSnapshot>> {
        let namespace = request.namespace;
        let build_snapshots = register
            .list_resource::<BuildSnapshot>(Some(namespace.as_str()), None)
            .await?;
        let snapshots: Vec<models::BuildSnapshot> = build_snapshots
            .into_iter()
            .map(|(key, build_snapshot)| models::BuildSnapshot {
                id: remove_resource_namespace::<BuildSnapshot>(key.as_str(), namespace.as_str())
                    .to_owned(),
                latest_version: build_snapshot.latest_version,
            })
            .collect();
        Ok(snapshots)
    }

    pub async fn delete_build_snapshot(
        mut register: Register,
        request: models::DeleteBuildSnapshotRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match validations::validate_delete_build_snapshot_request(&mut register, &request).await {
            Ok(()) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_delete_build_snapshot(&mut register, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_delete_build_snapshot(
        register: &mut Register,
        request: models::DeleteBuildSnapshotRequest,
    ) -> pipebuilder_common::Result<models::DeleteBuildSnapshotResponse> {
        let namespace = request.namespace;
        let id = request.id;
        register
            .delete_resource::<BuildSnapshot>(Some(namespace.as_str()), id.as_str(), None)
            .await?;
        Ok(models::DeleteBuildSnapshotResponse {})
    }

    pub async fn get_build_metadata(
        mut register: Register,
        lease_id: i64,
        request: models::GetBuildRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_get_build_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        let response = match do_get_build_metadata(&mut register, lease_id, request).await {
            Ok(response) => response,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        match response {
            Some(response) => Ok(utils::handlers::ok(&response)),
            None => Ok(utils::handlers::http_not_found(Failure::new(String::from(
                "build metadata not found",
            )))),
        }
    }

    async fn do_get_build_metadata(
        register: &mut Register,
        lease_id: i64,
        request: models::GetBuildRequest,
    ) -> pipebuilder_common::Result<Option<models::BuildMetadata>> {
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        let build_metadata = register
            .get_resource::<BuildMetadata>(
                Some(namespace.as_str()),
                id.as_str(),
                Some(version),
                lease_id,
            )
            .await?;
        Ok(build_metadata.map(|b| models::BuildMetadata {
            id,
            version,
            target_platform: b.target_platform,
            status: b.status,
            timestamp: b.timestamp,
            builder_id: b.builder_id,
            builder_address: b.builder_address,
            message: b.message,
        }))
    }

    pub async fn list_build_metadata(
        mut register: Register,
        request: models::ListBuildRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_list_build_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_list_build_metadata(&mut register, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_list_build_metadata(
        register: &mut Register,
        request: models::ListBuildRequest,
    ) -> pipebuilder_common::Result<Vec<models::BuildMetadata>> {
        let namespace = request.namespace;
        let id = request.id.as_deref();
        let build_metadatas = register
            .list_resource::<BuildMetadata>(Some(namespace.as_str()), id)
            .await?;
        let build_metadatas = build_metadatas
            .into_iter()
            .map(|(key, build_metadata)| {
                let id_version =
                    remove_resource_namespace::<BuildMetadata>(key.as_str(), namespace.as_str());
                let id_version = id_version.split('/').collect::<Vec<&str>>();
                let id = id_version.get(0).expect("id not found in key").to_string();
                let version: u64 = id_version
                    .get(1)
                    .expect("version not found in key")
                    .parse()
                    .expect("cannot parse version as u64");
                models::BuildMetadata {
                    id,
                    version,
                    target_platform: build_metadata.target_platform,
                    status: build_metadata.status,
                    timestamp: build_metadata.timestamp,
                    builder_id: build_metadata.builder_id,
                    builder_address: build_metadata.builder_address,
                    message: build_metadata.message,
                }
            })
            .collect::<Vec<models::BuildMetadata>>();
        Ok(build_metadatas)
    }

    pub async fn cancel_build(
        mut register: Register,
        lease_id: i64,
        request: models::CancelBuildRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_cancel_build_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        let request_clone = request.clone();
        // query version build for builder address
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        let build_metadata = match do_get_build_metadata(
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
            Ok(build_metadata) => build_metadata,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        let build_metadata = match build_metadata {
            Some(build_metadata) => build_metadata,
            None => {
                return Ok(utils::handlers::http_not_found(Failure::new(format!(
                    "build metadata (namespace = {}, id = {}, build_version = {}) not found",
                    namespace, id, version
                ))))
            }
        };
        // cancel local build at builder
        let builder_id = build_metadata.builder_id;
        let builder_address = build_metadata.builder_address;
        info!(
            builder_id = builder_id.as_str(),
            builder_address = builder_address.as_str(),
            "cancel build at builder",
        );
        let mut builder_client =
            match utils::handlers::builder_client(builder_address.as_str()).await {
                Ok(builder_client) => builder_client,
                Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
            };
        match do_cancel_build(&mut builder_client, request_clone).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_cancel_build(
        client: &mut BuilderClient<Channel>,
        request: models::CancelBuildRequest,
    ) -> pipebuilder_common::Result<models::CancelBuildResponse> {
        let request: CancelBuildRequest = request.into();
        let resp = client.cancel_build(request).await?;
        Ok(resp.into_inner().into())
    }

    pub async fn delete_build_metadata(
        mut register: Register,
        lease_id: i64,
        request: models::DeleteBuildRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_delete_build_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        let namespace = request.namespace.as_str();
        let id = request.id.as_str();
        let version = request.version;
        let build_metadata = match register
            .get_resource::<BuildMetadata>(Some(namespace), id, Some(version), lease_id)
            .await
        {
            Ok(build_metadata) => build_metadata,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        let build_metadata = match build_metadata {
            Some(build_metadata) => build_metadata,
            None => {
                return Ok(utils::handlers::http_bad_request(Failure::new(format!(
                    "build {}/{}/{} not found",
                    namespace, id, version
                ))))
            }
        };
        if !build_metadata.is_stopped() {
            return Ok(utils::handlers::http_bad_request(Failure::new(format!(
                "build {}/{}/{} is running, cancel required before delete",
                namespace, id, version
            ))));
        }
        match do_delete_build(&mut register, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
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
            .delete_resource::<BuildMetadata>(Some(namespace.as_str()), id.as_str(), Some(version))
            .await?;
        Ok(models::DeleteBuildResponse {})
    }

    pub async fn get_build_log(
        mut register: Register,
        lease_id: i64,
        request: models::GetBuildLogRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_get_build_log_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        let request_clone = request.clone();
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        let build_metadata = match do_get_build_metadata(
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
            Ok(build_metadata) => build_metadata,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        let build_metadata = match build_metadata {
            Some(build_metadata) => build_metadata,
            None => {
                return Ok(utils::handlers::http_not_found(Failure::new(format!(
                    "build metadata {}/{}/{} not found",
                    namespace, id, version
                ))))
            }
        };
        // get local build log at builder
        let builder_id = build_metadata.builder_id;
        let builder_address = build_metadata.builder_address;
        info!(
            builder_id = builder_id.as_str(),
            builder_address = builder_address.as_str(),
            "get build log at builder",
        );
        let mut builder_client =
            match utils::handlers::builder_client(builder_address.as_str()).await {
                Ok(builder_client) => builder_client,
                Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
            };
        match do_get_build_log(&mut builder_client, request_clone).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_get_build_log(
        client: &mut BuilderClient<Channel>,
        request: models::GetBuildLogRequest,
    ) -> pipebuilder_common::Result<models::GetBuildLogResponse> {
        let request: GetBuildLogRequest = request.into();
        let resp = client.get_build_log(request).await?;
        Ok(resp.into_inner().into())
    }

    pub async fn scan_build(
        mut register: Register,
        lease_id: i64,
        request: models::ScanBuildRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        let builder_id = request.builder_id.as_str();
        let node_state =
            match utils::handlers::get_internal_node_state(&mut register, lease_id, builder_id)
                .await
            {
                Ok(node_state) => node_state,
                Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
            };

        // find builder address
        let node_state = match node_state {
            Some(node_state) => node_state,
            None => {
                return Ok(utils::handlers::http_not_found(Failure::new(format!(
                    "node '{}' not found",
                    builder_id
                ))))
            }
        };
        match validations::validate_node_state(&node_state, &NodeRole::Builder) {
            Ok(()) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        }
        let address = node_state.external_address;
        let mut client = match utils::handlers::builder_client(address.as_str()).await {
            Ok(client) => client,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        match do_scan_build(&mut client, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_scan_build(
        client: &mut BuilderClient<Channel>,
        request: models::ScanBuildRequest,
    ) -> pipebuilder_common::Result<Vec<models::BuildMetadataKey>> {
        let request: ScanBuildRequest = request.into();
        let response = client.scan_build(request).await?;
        let response = response.into_inner();
        let builds = response.builds;
        let builds = builds
            .into_iter()
            .map(|b| b.into())
            .collect::<Vec<models::BuildMetadataKey>>();
        Ok(builds)
    }

    pub async fn scan_build_cache(
        mut register: Register,
        lease_id: i64,
        request: models::ScanBuildCacheRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        let builder_id = request.builder_id.as_str();
        let node_state =
            match utils::handlers::get_internal_node_state(&mut register, lease_id, builder_id)
                .await
            {
                Ok(node_state) => node_state,
                Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
            };
        let node_state = match node_state {
            Some(node_state) => node_state,
            None => {
                return Ok(utils::handlers::http_not_found(Failure::new(format!(
                    "node '{}' not found",
                    builder_id
                ))))
            }
        };
        match validations::validate_node_state(&node_state, &NodeRole::Builder) {
            Ok(()) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        }
        // find builder address
        let address = node_state.external_address;
        let mut client = match utils::handlers::builder_client(address.as_str()).await {
            Ok(client) => client,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        match do_scan_build_cache(&mut client, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_scan_build_cache(
        client: &mut BuilderClient<Channel>,
        request: models::ScanBuildCacheRequest,
    ) -> pipebuilder_common::Result<Vec<models::BuildCacheMetadata>> {
        let request: ScanBuildCacheRequest = request.into();
        let response = client.scan_build_cache(request).await?;
        let response = response.into_inner();
        let caches = response.caches;
        let caches = caches
            .into_iter()
            .map(|c| c.into())
            .collect::<Vec<models::BuildCacheMetadata>>();
        Ok(caches)
    }

    pub async fn delete_build_cache(
        mut register: Register,
        lease_id: i64,
        request: models::DeleteBuildCacheRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        let builder_id = request.builder_id.as_str();
        let node_state =
            match utils::handlers::get_internal_node_state(&mut register, lease_id, builder_id)
                .await
            {
                Ok(node_state) => node_state,
                Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
            };
        let node_state = match node_state {
            Some(node_state) => node_state,
            None => {
                return Ok(utils::handlers::http_not_found(Failure::new(format!(
                    "node '{}' not found",
                    builder_id
                ))))
            }
        };
        match validations::validate_node_state(&node_state, &NodeRole::Builder) {
            Ok(()) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        }
        // find builder address
        let address = node_state.external_address;
        let mut client = match utils::handlers::builder_client(address.as_str()).await {
            Ok(client) => client,
            Err(err) => return Ok(utils::handlers::http_internal_error(err.into())),
        };
        match do_delete_build_cache(&mut client, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_delete_build_cache(
        client: &mut BuilderClient<Channel>,
        request: models::DeleteBuildCacheRequest,
    ) -> pipebuilder_common::Result<models::DeleteBuildCacheResponse> {
        let request: DeleteBuildCacheRequest = request.into();
        let response = client.delete_build_cache(request).await?;
        Ok(response.into_inner().into())
    }
}
