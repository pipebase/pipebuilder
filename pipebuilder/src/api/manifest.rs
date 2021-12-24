pub mod filters {
    use super::handlers;
    use crate::utils;
    use pipebuilder_common::{
        api::models, grpc::repository::repository_client::RepositoryClient, Register,
    };
    use tonic::transport::Channel;
    use warp::Filter;

    // manifest api
    pub fn v1_manifest(
        repository_client: RepositoryClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        v1_manifest_post(repository_client.clone(), register.clone())
            .or(v1_manifest_get(repository_client.clone(), register.clone()))
            .or(v1_manifest_metadata_list(register.clone()))
            .or(v1_manifest_delete(repository_client, register.clone()))
            .or(v1_manifest_snapshot_list(register.clone()))
            .or(v1_manifest_snapshot_delete(register))
    }

    pub fn v1_manifest_post(
        repository_client: RepositoryClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "manifest")
            .and(warp::post())
            .and(utils::filters::with_repository_client(repository_client))
            .and(utils::filters::with_register(register))
            .and(utils::filters::json_request::<models::PostManifestRequest>())
            .and_then(handlers::post_manifest)
    }

    pub fn v1_manifest_get(
        repository_client: RepositoryClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "manifest")
            .and(warp::get())
            .and(utils::filters::with_repository_client(repository_client))
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::GetManifestRequest>())
            .and_then(handlers::get_manifest)
    }

    pub fn v1_manifest_snapshot_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "manifest" / "snapshot")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::ListManifestSnapshotRequest>())
            .and_then(handlers::list_manifest_snapshot)
    }

    pub fn v1_manifest_snapshot_delete(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "manifest" / "snapshot")
            .and(warp::delete())
            .and(utils::filters::with_register(register))
            .and(utils::filters::json_request::<
                models::DeleteManifestSnapshotRequest,
            >())
            .and_then(handlers::delete_manifest_snapshot)
    }

    pub fn v1_manifest_metadata_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "manifest" / "metadata")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::ListManifestMetadataRequest>())
            .and_then(handlers::list_manifest_metadata)
    }

    pub fn v1_manifest_delete(
        repository_client: RepositoryClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "manifest")
            .and(warp::delete())
            .and(utils::filters::with_repository_client(repository_client))
            .and(utils::filters::with_register(register))
            .and(utils::filters::json_request::<models::DeleteManifestRequest>())
            .and_then(handlers::delete_manifest)
    }
}

mod handlers {
    use crate::{utils, validations};
    use pipebuilder_common::{
        api::models,
        grpc::repository::{
            repository_client::RepositoryClient, DeleteManifestRequest, GetManifestRequest,
            PutManifestRequest,
        },
        remove_resource_namespace, ManifestMetadata, ManifestSnapshot, Register,
    };
    use std::convert::Infallible;
    use tonic::transport::Channel;

    pub async fn post_manifest(
        mut client: RepositoryClient<Channel>,
        mut register: Register,
        request: models::PostManifestRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_post_manifest_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_post_manifest(&mut client, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_post_manifest(
        client: &mut RepositoryClient<Channel>,
        request: models::PostManifestRequest,
    ) -> pipebuilder_common::Result<models::PostManifestResponse> {
        let request: PutManifestRequest = request.into();
        let response = client.put_manifest(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn get_manifest(
        mut client: RepositoryClient<Channel>,
        mut register: Register,
        request: models::GetManifestRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_get_manifest_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_get_manifest(&mut client, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_not_found(err.into())),
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
        // validate request
        match validations::validate_list_manifest_snapshot_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_list_manifest_snapshot(&mut register, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_list_manifest_snapshot(
        register: &mut Register,
        request: models::ListManifestSnapshotRequest,
    ) -> pipebuilder_common::Result<Vec<models::ManifestSnapshot>> {
        let namespace = request.namespace;
        let manifest_snapshots = register
            .list_resource::<ManifestSnapshot>(Some(namespace.as_str()), None)
            .await?;
        let snapshots: Vec<models::ManifestSnapshot> = manifest_snapshots
            .into_iter()
            .map(|(key, manifest_snapshot)| models::ManifestSnapshot {
                id: remove_resource_namespace::<ManifestSnapshot>(key.as_str(), namespace.as_str())
                    .to_owned(),
                latest_version: manifest_snapshot.latest_version,
            })
            .collect();
        Ok(snapshots)
    }

    pub async fn delete_manifest_snapshot(
        mut register: Register,
        request: models::DeleteManifestSnapshotRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match validations::validate_delete_manifest_snapshot_request(&mut register, &request).await
        {
            Ok(()) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_delete_manifest_snapshot(&mut register, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_delete_manifest_snapshot(
        register: &mut Register,
        request: models::DeleteManifestSnapshotRequest,
    ) -> pipebuilder_common::Result<models::DeleteManifestSnapshotResponse> {
        let namespace = request.namespace;
        let id = request.id;
        register
            .delete_resource::<ManifestSnapshot>(Some(namespace.as_str()), id.as_str(), None)
            .await?;
        Ok(models::DeleteManifestSnapshotResponse {})
    }

    pub async fn delete_manifest(
        mut client: RepositoryClient<Channel>,
        mut register: Register,
        request: models::DeleteManifestRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_delete_manifest_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_delete_manifest(&mut client, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
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

    pub async fn list_manifest_metadata(
        mut register: Register,
        request: models::ListManifestMetadataRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_list_manifest_metadata_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_list_manifest_metadata(&mut register, request).await {
            Ok(resp) => Ok(utils::handlers::ok(&resp)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_list_manifest_metadata(
        register: &mut Register,
        request: models::ListManifestMetadataRequest,
    ) -> pipebuilder_common::Result<Vec<models::ManifestMetadata>> {
        let namespace = request.namespace.as_str();
        let id = request.id.as_deref();
        let metas = register
            .list_resource::<ManifestMetadata>(Some(namespace), id)
            .await?;
        let metas = metas
            .into_iter()
            .map(|(key, meta)| {
                let id_version =
                    remove_resource_namespace::<ManifestMetadata>(key.as_str(), namespace);
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
}
