use pipebuilder_common::{
    api::{
        client::ApiClient,
        models::{ListNodeStateRequest, NodeState},
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
