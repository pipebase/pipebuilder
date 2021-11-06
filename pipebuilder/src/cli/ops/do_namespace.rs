use super::{
    do_project::{delete_project, list_project},
    print::Printer,
};
use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{DeleteNamespaceRequest, ListNamespaceRequest, Namespace, UpdateNamespaceRequest},
    },
    Result,
};

pub(crate) async fn create_namespace(client: &ApiClient, id: String) -> Result<Namespace> {
    let request = UpdateNamespaceRequest { id };
    let namespace = client.update_namespace(&request).await?;
    Ok(namespace)
}

pub(crate) async fn list_namespace(client: &ApiClient) -> Result<Vec<Namespace>> {
    let request = ListNamespaceRequest {};
    let namespaces = client.list_namespace(&request).await?;
    Ok(namespaces)
}

pub(crate) async fn delete_namespace(client: &ApiClient, namespace: String) -> Result<()> {
    let mut printer = Printer::new();
    for project in list_project(client, namespace.clone()).await? {
        let id = project.id;
        delete_project(client, namespace.clone(), id).await?;
    }
    printer.status("Deleting", format!("namespace {}", namespace))?;
    let request = DeleteNamespaceRequest { id: namespace };
    client.delete_namespace(&request).await
}
