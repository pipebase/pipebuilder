pub mod filters {

    use super::handlers;
    use crate::utils;
    use pipebuilder_common::{api::models, Register};
    use warp::Filter;

    // project api
    pub fn v1_project(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        v1_project_put(register.clone(), lease_id)
            .or(v1_project_delete(register.clone()))
            .or(v1_project_list(register))
    }

    pub fn v1_project_put(
        register: Register,
        lease_id: i64,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "project")
            .and(warp::post())
            .and(utils::filters::with_register(register))
            .and(utils::filters::with_lease_id(lease_id))
            .and(utils::filters::json_request::<models::UpdateProjectRequest>())
            .and_then(handlers::put_project)
    }

    pub fn v1_project_delete(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "project")
            .and(warp::delete())
            .and(utils::filters::with_register(register))
            .and(utils::filters::json_request::<models::DeleteProjectRequest>())
            .and_then(handlers::delete_project)
    }

    pub fn v1_project_list(
        register: Register,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "project")
            .and(warp::get())
            .and(utils::filters::with_register(register))
            .and(warp::query::<models::ListProjectRequest>())
            .and_then(handlers::list_project)
    }
}

mod handlers {
    use crate::{utils, validations};
    use pipebuilder_common::{api::models, remove_resource_namespace, Project, Register};
    use std::convert::Infallible;

    pub async fn put_project(
        mut register: Register,
        lease_id: i64,
        request: models::UpdateProjectRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_put_project_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_put_project(&mut register, lease_id, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
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
            .update_default_resource::<Project>(Some(namespace.as_str()), id.as_str(), lease_id)
            .await?;
        let created = project.created;
        Ok(models::Project { id, created })
    }

    pub async fn list_project(
        mut register: Register,
        request: models::ListProjectRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_list_project_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_list_project(&mut register, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_list_project(
        register: &mut Register,
        request: models::ListProjectRequest,
    ) -> pipebuilder_common::Result<Vec<models::Project>> {
        let namespace = request.namespace;
        let projects = register
            .list_resource::<Project>(Some(namespace.as_str()), None)
            .await?;
        let projects = projects
            .into_iter()
            .map(|(key, project)| {
                let id = remove_resource_namespace::<Project>(key.as_str(), namespace.as_str());
                models::Project {
                    id: id.to_owned(),
                    created: project.created,
                }
            })
            .collect::<Vec<models::Project>>();
        Ok(projects)
    }

    pub async fn delete_project(
        mut register: Register,
        request: models::DeleteProjectRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // validate request
        match validations::validate_delete_project_request(&mut register, &request).await {
            Ok(_) => (),
            Err(err) => return Ok(utils::handlers::http_bad_request(err.into())),
        };
        match do_delete_project(&mut register, request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_internal_error(err.into())),
        }
    }

    async fn do_delete_project(
        register: &mut Register,
        request: models::DeleteProjectRequest,
    ) -> pipebuilder_common::Result<models::DeleteProjectResponse> {
        let namespace = request.namespace;
        let id = request.id;
        register
            .delete_resource::<Project>(Some(namespace.as_str()), id.as_str(), None)
            .await?;
        Ok(models::DeleteProjectResponse {})
    }
}
