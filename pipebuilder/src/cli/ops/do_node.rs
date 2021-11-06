use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{ActivateNodeRequest, DeactivateNodeRequest, ListNodeStateRequest, NodeState},
    },
    NodeRole, Result,
};

pub(crate) async fn list_node_state(
    client: &ApiClient,
    role: Option<NodeRole>,
) -> Result<Vec<NodeState>> {
    let request = ListNodeStateRequest { role };
    client.list_node_state(&request).await
}

pub(crate) async fn activate_node(client: &ApiClient, role: NodeRole, id: String) -> Result<()> {
    let request = ActivateNodeRequest { role, id };
    let _ = client.activate_node(&request).await?;
    Ok(())
}

pub(crate) async fn deactivate_node(client: &ApiClient, role: NodeRole, id: String) -> Result<()> {
    let request = DeactivateNodeRequest { role, id };
    let _ = client.deactivate_node(&request).await?;
    Ok(())
}
