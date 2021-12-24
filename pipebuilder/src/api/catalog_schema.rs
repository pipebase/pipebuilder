pub mod filters {
    use super::handlers;
    use crate::utils;
    use pipebuilder_common::{
        api::models, grpc::repository::repository_client::RepositoryClient, Register,
    };
    use tonic::transport::Channel;
    use warp::Filter;

    // catalog schema api
    pub fn v1_catalog_schema(
        repository_client: RepositoryClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        v1_catalog_schema_post(repository_client.clone(), register.clone())
            .or(v1_catalog_schema_get(
                repository_client.clone(),
                register.clone(),
            ))
            .or(v1_catalog_schema_metadata_list(register.clone()))
            .or(v1_catalog_schema_delete(
                repository_client,
                register.clone(),
            ))
            .or(v1_catalog_schema_snapshot_list(register.clone()))
            .or(v1_catalog_schema_snapshot_delete(register))
    }

    pub fn v1_catalog_schema_post(
        repository_client: RepositoryClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "catalog-schema")
            .and(warp::post())
            .and(utils::filters::with_repository_client(repository_client))
            .and(utils::filters::with_register(register))
            .and(utils::filters::json_request::<
                models::PostCatalogSchemaRequest,
            >())
            .and_then(handlers::post_catalog_schema)
    }

    pub fn v1_catalog_schema_get(
        repository_client: RepositoryClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "catalog-schema")
            .and(warp::get())
            .and(utils::filters::with_repository_client(repository_client))
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::GetCatalogSchemaRequest>())
            .and_then(handlers::get_catalog_schema)
    }

    pub fn v1_catalog_schema_snapshot_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "catalog-schema" / "snapshot")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::ListCatalogSchemaSnapshotRequest>())
            .and_then(handlers::list_catalog_schema_snapshot)
    }

    pub fn v1_catalog_schema_snapshot_delete(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "catalog-schema" / "snapshot")
            .and(warp::delete())
            .and(utils::filters::with_register(register))
            .and(utils::filters::json_request::<
                models::DeleteCatalogSchemaSnapshotRequest,
            >())
            .and_then(handlers::delete_catalog_schema_snapshot)
    }

    pub fn v1_catalog_schema_metadata_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "catalog-schema" / "metadata")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::ListCatalogSchemaMetadataRequest>())
            .and_then(handlers::list_catalog_schema_metadata)
    }

    pub fn v1_catalog_schema_delete(
        repository_client: RepositoryClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "catalog-schema")
            .and(warp::delete())
            .and(utils::filters::with_repository_client(repository_client))
            .and(utils::filters::with_register(register))
            .and(utils::filters::json_request::<
                models::DeleteCatalogSchemaRequest,
            >())
            .and_then(handlers::delete_catalog_schema)
    }
}

mod handlers {
    use crate::{utils, validations};
    use pipebuilder_common::{
        api::models,
        grpc::repository::{
            repository_client::RepositoryClient, DeleteCatalogSchemaRequest,
            GetCatalogSchemaRequest, PutCatalogSchemaRequest,
        },
        remove_resource_namespace, CatalogSchemaMetadata, CatalogSchemaSnapshot, Register,
    };
    use std::convert::Infallible;
    use tonic::transport::Channel;

    pub async fn post_catalog_schema(
        mut client: RepositoryClient<Channel>,
        mut register: Register,
        request: models::PostCatalogSchemaRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_post_catalog_schema_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_post_catalog_schema(&mut client, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_post_catalog_schema(
        client: &mut RepositoryClient<Channel>,
        request: models::PostCatalogSchemaRequest,
    ) -> pipebuilder_common::Result<models::PostCatalogSchemaResponse> {
        let request: PutCatalogSchemaRequest = request.into();
        let response = client.put_catalog_schema(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn get_catalog_schema(
        mut client: RepositoryClient<Channel>,
        mut register: Register,
        request: models::GetCatalogSchemaRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_get_catalog_schema_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_get_catalog_schema(&mut client, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_not_found(err.into())),
        }
    }

    async fn do_get_catalog_schema(
        client: &mut RepositoryClient<Channel>,
        request: models::GetCatalogSchemaRequest,
    ) -> pipebuilder_common::Result<models::GetCatalogSchemaResponse> {
        let request: GetCatalogSchemaRequest = request.into();
        let response = client.get_catalog_schema(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn list_catalog_schema_snapshot(
        mut register: Register,
        request: models::ListCatalogSchemaSnapshotRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_list_catalog_schema_snapshot_request(&mut register, &request)
            .await
        {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_list_catalog_schema_snapshot(&mut register, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_list_catalog_schema_snapshot(
        register: &mut Register,
        request: models::ListCatalogSchemaSnapshotRequest,
    ) -> pipebuilder_common::Result<Vec<models::CatalogSchemaSnapshot>> {
        let namespace = request.namespace;
        let manifest_snapshots = register
            .list_resource::<CatalogSchemaSnapshot>(Some(namespace.as_str()), None)
            .await?;
        let snapshots: Vec<models::CatalogSchemaSnapshot> = manifest_snapshots
            .into_iter()
            .map(|(key, manifest_snapshot)| models::CatalogSchemaSnapshot {
                id: remove_resource_namespace::<CatalogSchemaSnapshot>(
                    key.as_str(),
                    namespace.as_str(),
                )
                .to_owned(),
                latest_version: manifest_snapshot.latest_version,
            })
            .collect();
        Ok(snapshots)
    }

    pub async fn delete_catalog_schema_snapshot(
        mut register: Register,
        request: models::DeleteCatalogSchemaSnapshotRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match validations::validate_delete_catalog_schema_snapshot_request(&mut register, &request)
            .await
        {
            Ok(()) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_delete_catalog_schema_snapshot(&mut register, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_delete_catalog_schema_snapshot(
        register: &mut Register,
        request: models::DeleteCatalogSchemaSnapshotRequest,
    ) -> pipebuilder_common::Result<models::DeleteCatalogSchemaSnapshotResponse> {
        let namespace = request.namespace;
        let id = request.id;
        register
            .delete_resource::<CatalogSchemaSnapshot>(Some(namespace.as_str()), id.as_str(), None)
            .await?;
        Ok(models::DeleteCatalogSchemaSnapshotResponse {})
    }

    pub async fn delete_catalog_schema(
        mut client: RepositoryClient<Channel>,
        mut register: Register,
        request: models::DeleteCatalogSchemaRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_delete_catalog_schema_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_delete_catalog_schema(&mut client, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_delete_catalog_schema(
        client: &mut RepositoryClient<Channel>,
        request: models::DeleteCatalogSchemaRequest,
    ) -> pipebuilder_common::Result<models::DeleteCatalogSchemaResponse> {
        let request: DeleteCatalogSchemaRequest = request.into();
        let response = client.delete_catalog_schema(request).await?;
        Ok(response.into_inner().into())
    }

    pub async fn list_catalog_schema_metadata(
        mut register: Register,
        request: models::ListCatalogSchemaMetadataRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_list_catalog_schema_metadata_request(&mut register, &request)
            .await
        {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_list_catalog_schema_metadata(&mut register, request).await {
            Ok(resp) => Ok(utils::handlers::ok(&resp)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_list_catalog_schema_metadata(
        register: &mut Register,
        request: models::ListCatalogSchemaMetadataRequest,
    ) -> pipebuilder_common::Result<Vec<models::CatalogSchemaMetadata>> {
        let namespace = request.namespace.as_str();
        let id = request.id.as_deref();
        let metas = register
            .list_resource::<CatalogSchemaMetadata>(Some(namespace), id)
            .await?;
        let metas = metas
            .into_iter()
            .map(|(key, meta)| {
                let id_version =
                    remove_resource_namespace::<CatalogSchemaMetadata>(key.as_str(), namespace);
                let id_version = id_version.split('/').collect::<Vec<&str>>();
                let id = id_version.get(0).expect("id not found in key").to_string();
                let version: u64 = id_version
                    .get(1)
                    .expect("version not found in key")
                    .parse()
                    .expect("cannot parse version as u64");
                models::CatalogSchemaMetadata {
                    id,
                    version,
                    pulls: meta.pulls,
                    size: meta.size,
                    created: meta.created,
                }
            })
            .collect::<Vec<models::CatalogSchemaMetadata>>();
        Ok(metas)
    }
}
