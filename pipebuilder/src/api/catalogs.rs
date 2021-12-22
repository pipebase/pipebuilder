pub mod filters {
    use super::handlers;
    use crate::utils;
    use pipebuilder_common::{
        api::models, grpc::repository::repository_client::RepositoryClient, Register,
    };
    use tonic::transport::Channel;
    use warp::Filter;

    // catalogs api
    pub fn v1_catalogs(
        repository_client: RepositoryClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        v1_catalogs_post(repository_client.clone(), register.clone())
            .or(v1_catalogs_get(repository_client.clone(), register.clone()))
            .or(v1_catalogs_metadata_list(register.clone()))
            .or(v1_catalogs_delete(repository_client, register.clone()))
            .or(v1_catalogs_snapshot_list(register.clone()))
            .or(v1_catalogs_snapshot_delete(register))
    }

    pub fn v1_catalogs_post(
        repository_client: RepositoryClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "catalogs")
            .and(warp::post())
            .and(utils::filters::with_repository_client(repository_client))
            .and(utils::filters::with_register(register))
            .and(utils::filters::json_request::<models::PostCatalogsRequest>())
            .and_then(handlers::post_catalogs)
    }

    pub fn v1_catalogs_get(
        repository_client: RepositoryClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "catalogs")
            .and(warp::get())
            .and(utils::filters::with_repository_client(repository_client))
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::GetCatalogsRequest>())
            .and_then(handlers::get_catalogs)
    }

    pub fn v1_catalogs_snapshot_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "catalogs" / "snapshot")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::ListCatalogsSnapshotRequest>())
            .and_then(handlers::list_catalogs_snapshot)
    }

    pub fn v1_catalogs_snapshot_delete(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "catalogs" / "snapshot")
            .and(warp::delete())
            .and(utils::filters::with_register(register))
            .and(utils::filters::json_request::<
                models::DeleteCatalogsSnapshotRequest,
            >())
            .and_then(handlers::delete_catalogs_snapshot)
    }

    pub fn v1_catalogs_metadata_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "catalogs" / "metadata")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::ListCatalogsMetadataRequest>())
            .and_then(handlers::list_catalogs_metadata)
    }

    pub fn v1_catalogs_delete(
        repository_client: RepositoryClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "catalogs")
            .and(warp::delete())
            .and(utils::filters::with_repository_client(repository_client))
            .and(utils::filters::with_register(register))
            .and(utils::filters::json_request::<models::DeleteCatalogsRequest>())
            .and_then(handlers::delete_catalogs)
    }
}

mod handlers {
    use crate::{utils, validations};
    use pipebuilder_common::{
        api::models,
        grpc::repository::{
            repository_client::RepositoryClient, DeleteCatalogsRequest, GetCatalogsRequest,
            PutCatalogsRequest,
        },
        remove_resource_namespace, CatalogsMetadata, CatalogsSnapshot, Register,
    };
    use std::convert::Infallible;
    use tonic::transport::Channel;

    pub async fn post_catalogs(
        mut client: RepositoryClient<Channel>,
        mut register: Register,
        request: models::PostCatalogsRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_post_catalogs_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_post_catalogs(&mut client, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_post_catalogs(
        client: &mut RepositoryClient<Channel>,
        request: models::PostCatalogsRequest,
    ) -> pipebuilder_common::Result<models::PostCatalogsResponse> {
        let request: PutCatalogsRequest = request.into();
        let response = client.put_catalogs(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn get_catalogs(
        mut client: RepositoryClient<Channel>,
        mut register: Register,
        request: models::GetCatalogsRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_get_catalogs_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_get_catalogs(&mut client, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_not_found(err.into())),
        }
    }

    async fn do_get_catalogs(
        client: &mut RepositoryClient<Channel>,
        request: models::GetCatalogsRequest,
    ) -> pipebuilder_common::Result<models::GetCatalogsResponse> {
        let request: GetCatalogsRequest = request.into();
        let response = client.get_catalogs(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn list_catalogs_snapshot(
        mut register: Register,
        request: models::ListCatalogsSnapshotRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_list_catalogs_snapshot_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_list_catalogs_snapshot(&mut register, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_list_catalogs_snapshot(
        register: &mut Register,
        request: models::ListCatalogsSnapshotRequest,
    ) -> pipebuilder_common::Result<Vec<models::CatalogsSnapshot>> {
        let namespace = request.namespace;
        let manifest_snapshots = register
            .list_resource::<CatalogsSnapshot>(Some(namespace.as_str()), None)
            .await?;
        let snapshots: Vec<models::CatalogsSnapshot> = manifest_snapshots
            .into_iter()
            .map(|(key, manifest_snapshot)| models::CatalogsSnapshot {
                id: remove_resource_namespace::<CatalogsSnapshot>(key.as_str(), namespace.as_str())
                    .to_owned(),
                latest_version: manifest_snapshot.latest_version,
            })
            .collect();
        Ok(snapshots)
    }

    pub async fn delete_catalogs_snapshot(
        mut register: Register,
        request: models::DeleteCatalogsSnapshotRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match validations::validate_delete_catalogs_snapshot_request(&mut register, &request).await
        {
            Ok(()) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_delete_catalogs_snapshot(&mut register, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_delete_catalogs_snapshot(
        register: &mut Register,
        request: models::DeleteCatalogsSnapshotRequest,
    ) -> pipebuilder_common::Result<models::DeleteCatalogsSnapshotResponse> {
        let namespace = request.namespace;
        let id = request.id;
        register
            .delete_resource::<CatalogsSnapshot>(Some(namespace.as_str()), id.as_str(), None)
            .await?;
        Ok(models::DeleteCatalogsSnapshotResponse {})
    }

    pub async fn delete_catalogs(
        mut client: RepositoryClient<Channel>,
        mut register: Register,
        request: models::DeleteCatalogsRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_delete_catalogs_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_delete_catalogs(&mut client, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_delete_catalogs(
        client: &mut RepositoryClient<Channel>,
        request: models::DeleteCatalogsRequest,
    ) -> pipebuilder_common::Result<models::DeleteCatalogsResponse> {
        let request: DeleteCatalogsRequest = request.into();
        let response = client.delete_catalogs(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn list_catalogs_metadata(
        mut register: Register,
        request: models::ListCatalogsMetadataRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_list_catalogs_metadata_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_list_catalogs_metadata(&mut register, request).await {
            Ok(resp) => Ok(utils::handlers::ok(&resp)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_list_catalogs_metadata(
        register: &mut Register,
        request: models::ListCatalogsMetadataRequest,
    ) -> pipebuilder_common::Result<Vec<models::CatalogsMetadata>> {
        let namespace = request.namespace.as_str();
        let id = request.id.as_deref();
        let metas = register
            .list_resource::<CatalogsMetadata>(Some(namespace), id)
            .await?;
        let metas = metas
            .into_iter()
            .map(|(key, meta)| {
                let id_version =
                    remove_resource_namespace::<CatalogsMetadata>(key.as_str(), namespace);
                let id_version = id_version.split('/').collect::<Vec<&str>>();
                let id = id_version.get(0).expect("id not found in key").to_string();
                let version: u64 = id_version
                    .get(1)
                    .expect("version not found in key")
                    .parse()
                    .expect("cannot parse version as u64");
                models::CatalogsMetadata {
                    id,
                    version,
                    pulls: meta.pulls,
                    size: meta.size,
                    created: meta.created,
                }
            })
            .collect::<Vec<models::CatalogsMetadata>>();
        Ok(metas)
    }
}
