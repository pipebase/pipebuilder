pub mod filters {
    use serde::de::DeserializeOwned;
    use std::path::PathBuf;
    use tokio::sync::mpsc;
    use warp::Filter;

    pub fn with_path(
        path: PathBuf,
    ) -> impl Filter<Extract = (PathBuf,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || path.clone())
    }

    pub fn with_shutdown_tx(
        tx: mpsc::Sender<()>,
    ) -> impl Filter<Extract = (mpsc::Sender<()>,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || tx.clone())
    }

    pub fn json_request<T>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone
    where
        T: Send + DeserializeOwned,
    {
        // When accepting a body, we want a JSON body and reject huge payloads
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }
}

pub mod handlers {

    use pipebuilder_common::api::models::Failure;

    use serde::Serialize;
    use warp::http::{Response, StatusCode};

    pub fn failure(status_code: StatusCode, failure: Failure) -> http::Result<Response<String>> {
        Response::builder()
            .status(status_code)
            .body(serde_json::to_string(&failure).unwrap())
    }

    #[allow(dead_code)]
    pub fn http_internal_error(f: Failure) -> http::Result<Response<String>> {
        failure(StatusCode::INTERNAL_SERVER_ERROR, f)
    }

    #[allow(dead_code)]
    pub fn http_service_unavailable(f: Failure) -> http::Result<Response<String>> {
        failure(StatusCode::SERVICE_UNAVAILABLE, f)
    }

    pub fn http_not_found(f: Failure) -> http::Result<Response<String>> {
        failure(StatusCode::NOT_FOUND, f)
    }

    #[allow(dead_code)]
    pub fn http_bad_request(f: Failure) -> http::Result<Response<String>> {
        failure(StatusCode::BAD_REQUEST, f)
    }

    pub fn ok<T>(t: &T) -> http::Result<Response<String>>
    where
        T: ?Sized + Serialize,
    {
        Response::builder()
            .status(StatusCode::OK)
            .body(serde_json::to_string::<T>(t).unwrap())
    }
}
