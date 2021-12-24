use super::{
    do_app::delete_app_all, do_build::delete_build_all, do_catalogs::delete_catalogs_all,
    do_manifest::delete_manifest_all, print::Printer,
};
use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{DeleteProjectRequest, ListProjectRequest, Project, UpdateProjectRequest},
    },
    Result,
};

pub(crate) async fn create_project(
    client: &ApiClient,
    namespace: String,
    id: String,
) -> Result<Project> {
    let request = UpdateProjectRequest { namespace, id };
    let project = client.update_project(&request).await?;
    Ok(project)
}

pub(crate) async fn list_project(client: &ApiClient, namespace: String) -> Result<Vec<Project>> {
    let request = ListProjectRequest { namespace };
    let projects = client.list_project(&request).await?;
    Ok(projects)
}

pub(crate) async fn delete_project(
    client: &ApiClient,
    namespace: String,
    id: String,
) -> Result<()> {
    let mut printer = Printer::new();
    printer.status(
        "Deleting",
        format!("project (namespace = {}, id = {})", namespace, id),
    )?;
    delete_app_all(client, namespace.clone(), id.clone()).await?;
    delete_build_all(client, namespace.clone(), id.clone()).await?;
    delete_manifest_all(client, namespace.clone(), id.clone()).await?;
    delete_catalogs_all(client, namespace.clone(), id.clone()).await?;
    let request = DeleteProjectRequest { namespace, id };
    client.delete_project(&request).await
}
