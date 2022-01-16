pub mod filters {
    use super::handlers;
    use crate::utils;
    use pipebuilder_common::api::models;
    use std::path::PathBuf;
    use warp::Filter;

    // catalogs api
    pub fn v1_catalogs(
        directory: PathBuf,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        v1_catalogs_get(directory)
    }

    pub fn v1_catalogs_get(
        directory: PathBuf,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1" / "catalogs")
            .and(warp::get())
            .and(utils::filters::with_path(directory))
            .and(warp::query::<models::GetCatalogsRequest>())
            .and_then(handlers::get_catalogs)
    }
}

pub mod handlers {
    use crate::utils;
    use pipebuilder_common::{api::models, read_file, PathBuilder};
    use std::{convert::Infallible, path};

    const PATH_CATALOGS: &str = "catalogs.yml";

    pub async fn get_catalogs(
        directory: path::PathBuf,
        request: models::GetCatalogsRequest,
    ) -> Result<impl warp::Reply, Infallible> {
        // TODO: validate request
        match do_get_catalogs(directory.as_path(), request).await {
            Ok(response) => Ok(utils::handlers::ok(&response)),
            Err(err) => Ok(utils::handlers::http_not_found(err.into())),
        }
    }

    async fn do_get_catalogs(
        directory: &path::Path,
        request: models::GetCatalogsRequest,
    ) -> pipebuilder_common::Result<models::GetCatalogsResponse> {
        let path = PathBuilder::default()
            .push(directory)
            .push(request.namespace)
            .push(request.id)
            .push(request.version.to_string())
            .push(PATH_CATALOGS)
            .build();
        let buffer = read_file(path.as_path()).await?;
        Ok(models::GetCatalogsResponse { buffer })
    }
}
