use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{ListProjectRequest, Project, UpdateProjectRequest},
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

pub(crate) async fn list_project(client: &ApiClient, namespace: &str) -> Result<Vec<Project>> {
    let request = ListProjectRequest {
        namespace: namespace.to_owned(),
    };
    let projects = client.list_project(&request).await?;
    Ok(projects)
}
