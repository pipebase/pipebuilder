pub mod filters {
    use super::handlers;
    use crate::utils;
    use pipebuilder_common::{
        api::models, grpc::repository::repository_client::RepositoryClient, Register,
    };
    use tonic::transport::Channel;
    use warp::Filter;

    // app api
    pub fn v1_app(
        repository_client: RepositoryClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        v1_app_get(repository_client.clone(), register.clone())
            .or(v1_app_metadata_list(register.clone()))
            .or(v1_app_delete(repository_client, register))
    }

    pub fn v1_app_get(
        repository_client: RepositoryClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "app")
            .and(warp::get())
            .and(utils::filters::with_repository_client(repository_client))
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::GetAppRequest>())
            .and_then(handlers::get_app)
    }

    pub fn v1_app_metadata_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "app" / "metadata")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::ListAppMetadataRequest>())
            .and_then(handlers::list_app_metadata)
    }

    pub fn v1_app_delete(
        repository_client: RepositoryClient<Channel>,
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "app")
            .and(warp::delete())
            .and(utils::filters::with_repository_client(repository_client))
            .and(utils::filters::with_register(register))
            .and(utils::filters::json_request::<models::DeleteAppRequest>())
            .and_then(handlers::delete_app)
    }
}

mod handlers {
    use crate::{utils, validations};
    use pipebuilder_common::{
        api::models,
        grpc::repository::{repository_client::RepositoryClient, DeleteAppRequest, GetAppRequest},
        remove_resource_namespace, AppMetadata, Register,
    };
    use std::convert::Infallible;
    use tonic::transport::Channel;

    pub async fn get_app(
        mut client: RepositoryClient<Channel>,
        mut register: Register,
        request: models::GetAppRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_get_app_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_get_app(&mut client, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_not_found(err.into())),
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
        mut register: Register,
        request: models::DeleteAppRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_delete_app_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_delete_app(&mut client, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
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

    pub async fn list_app_metadata(
        mut register: Register,
        request: models::ListAppMetadataRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_list_app_metadata_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_list_app_metadata(&mut register, request).await {
            Ok(resp) => Ok(utils::handlers::ok(&resp)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_list_app_metadata(
        register: &mut Register,
        request: models::ListAppMetadataRequest,
    ) -> pipebuilder_common::Result<Vec<models::AppMetadata>> {
        let namespace = request.namespace.as_str();
        let id = request.id.as_deref();
        let metas = register
            .list_resource::<AppMetadata>(Some(namespace), id)
            .await?;
        let metas = metas
            .into_iter()
            .map(|(key, meta)| {
                let id_version = remove_resource_namespace::<AppMetadata>(key.as_str(), namespace);
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
}
