pub mod filters {
    use crate::{admin, app, catalogs};
    use std::path::PathBuf;
    use tokio::sync::mpsc;
    use warp::Filter;

    pub fn api(
        app_directory: PathBuf,
        catalogs_directory: PathBuf,
        shutdown_tx: mpsc::Sender<()>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        app::filters::v1_app(app_directory)
            .or(catalogs::filters::v1_catalogs(catalogs_directory))
            .or(admin::filters::admin(shutdown_tx))
    }
}
