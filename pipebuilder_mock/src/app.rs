pub mod filters {
    use super::handlers;
    use crate::utils;
    use pipebuilder_common::api::models;
    use std::path::PathBuf;
    use warp::Filter;

    // app api
    pub fn v1_app(
        directory: PathBuf,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        v1_app_get(directory)
    }

    pub fn v1_app_get(
        directory: PathBuf,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "app")
            .and(warp::get())
            .and(utils::filters::with_path(directory))
            .and(warp::query::<models::GetAppRequest>())
            .and_then(handlers::get_app)
    }
}

mod handlers {

    use crate::utils;
    use pipebuilder_common::{api::models, read_file, PathBuilder};
    use std::{convert::Infallible, path};

    const PATH_APP: &str = "app";

    pub async fn get_app(
        directory: path::PathBuf,
        request: models::GetAppRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // TODO: validate request
        match do_get_app(directory.as_path(), request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_not_found(err.into())),
        }
    }

    async fn do_get_app(
        directory: &path::Path,
        request: models::GetAppRequest,
    ) -> pipebuilder_common::Result<models::GetAppResponse> {
        let path = PathBuilder::default()
            .push(directory)
            .push(request.namespace)
            .push(request.id)
            .push(request.build_version.to_string())
            .push(PATH_APP)
            .build();
        let buffer = read_file(path.as_path()).await?;
        Ok(models::GetAppResponse { buffer })
    }
}
