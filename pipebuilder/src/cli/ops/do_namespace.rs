use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{ListNamespaceRequest, Namespace, UpdateNamespaceRequest},
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
