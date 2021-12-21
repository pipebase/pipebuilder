pub mod filters {

    use super::handlers;
    use crate::utils;
    use pipebuilder_common::{api::models, Register};
    use warp::Filter;

    // namespace api
    pub fn v1_namespace(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        v1_namespace_put(register.clone(), lease_id)
            .or(v1_namespace_delete(register.clone()))
            .or(v1_namespace_list(register))
    }

    pub fn v1_namespace_put(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "namespace")
            .and(warp::post())
            .and(utils::filters::with_register(register))
            .and(utils::filters::with_lease_id(lease_id))
            .and(utils::filters::json_request::<models::UpdateNamespaceRequest>())
            .and_then(handlers::put_namespace)
    }

    pub fn v1_namespace_delete(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "namespace")
            .and(warp::delete())
            .and(utils::filters::with_register(register))
            .and(utils::filters::json_request::<models::DeleteNamespaceRequest>())
            .and_then(handlers::delete_namespace)
    }

    pub fn v1_namespace_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "namespace")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::ListNamespaceRequest>())
            .and_then(handlers::list_namespace)
    }
}

mod handlers {
    use crate::{utils, validations};
    use pipebuilder_common::{api::models, remove_resource, Namespace, Register};
    use std::convert::Infallible;

    pub async fn put_namespace(
        mut register: Register,
        lease_id: i64,
        request: models::UpdateNamespaceRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_put_namespace(&mut register, lease_id, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_put_namespace(
        register: &mut Register,
        lease_id: i64,
        request: models::UpdateNamespaceRequest,
    ) -> pipebuilder_common::Result<models::Namespace> {
        let id = request.id;
        let (_, namespace) = register
            .update_default_resource::<Namespace>(None, id.as_str(), lease_id)
            .await?;
        let created = namespace.created;
        Ok(models::Namespace { id, created })
    }

    pub async fn list_namespace(
        mut register: Register,
        _request: models::ListNamespaceRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        match do_list_namespace(&mut register, _request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_list_namespace(
        register: &mut Register,
        _request: models::ListNamespaceRequest,
    ) -> pipebuilder_common::Result<Vec<models::Namespace>> {
        let namespaces = register.list_resource::<Namespace>(None, None).await?;
        let namespaces = namespaces
            .into_iter()
            .map(|(key, namespace)| {
                let id = remove_resource::<Namespace>(key.as_str());
                models::Namespace {
                    id: id.to_owned(),
                    created: namespace.created,
                }
            })
            .collect::<Vec<models::Namespace>>();
        Ok(namespaces)
    }

    pub async fn delete_namespace(
        mut register: Register,
        request: models::DeleteNamespaceRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_delete_namespace_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_delete_namespace(&mut register, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_delete_namespace(
        register: &mut Register,
        request: models::DeleteNamespaceRequest,
    ) -> pipebuilder_common::Result<models::DeleteNamespaceResponse> {
        let id = request.id;
        register
            .delete_resource::<Namespace>(None, id.as_str(), None)
            .await?;
        Ok(models::DeleteNamespaceResponse {})
    }
}
