pub mod filters {
    use crate::{admin, app, build, catalog_schema, catalogs, manifest, namespace, node, project};
    use pipebuilder_common::{
        grpc::{
            repository::repository_client::RepositoryClient,
            schedule::scheduler_client::SchedulerClient,
        },
        NodeService, Register,
    };
    use tonic::transport::Channel;
    use warp::Filter;

    pub fn api(
        repository_client: RepositoryClient<Channel>,
        scheduler_client: SchedulerClient<Channel>,
        register: Register,
        lease_id: i64,
        node_svc: NodeService,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        app::filters::v1_app(repository_client.clone(), register.clone())
            .or(build::filters::v1_build(
                scheduler_client,
                register.clone(),
                lease_id,
            ))
            .or(manifest::filters::v1_manifest(
                repository_client.clone(),
                register.clone(),
            ))
            .or(catalogs::filters::v1_catalogs(
                repository_client.clone(),
                register.clone(),
            ))
            .or(catalog_schema::filters::v1_catalog_schema(
                repository_client,
                register.clone(),
            ))
            .or(namespace::filters::v1_namespace(register.clone(), lease_id))
            .or(node::filters::v1_node(register.clone(), lease_id))
            .or(project::filters::v1_project(register, lease_id))
            .or(admin::filters::admin(node_svc))
    }
}
