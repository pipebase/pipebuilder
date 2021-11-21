use std::path::Path;

use pipebuilder_common::{
    api::{
        client::{ApiClient, ApiClientConfig},
        models::{ListNodeStateRequest, NodeState, ShutdownNodeRequest, ShutdownRequest},
    },
    open_file, parse_config, NodeRole, Result,
};

pub async fn build_api_client<P>(path: P) -> Result<ApiClient>
where
    P: AsRef<Path>,
{
    let config_file = open_file(path).await?;
    let config = parse_config::<ApiClientConfig>(config_file).await?;
    Ok(config.into())
}

pub async fn shutdown_ci(client: &ApiClient) -> Result<()> {
    let node_states = client
        .list_node_state(&ListNodeStateRequest { role: None })
        .await?;
    for node_state in node_states {
        let role = node_state.role;
        let id = node_state.id;
        match role {
            NodeRole::Api => {
                client.shutdown(&ShutdownRequest {}).await?;
            }
            _ => {
                client
                    .shutdown_node(&ShutdownNodeRequest { role, id })
                    .await?;
            }
        };
    }
    Ok(())
}

pub async fn list_node_state(client: &ApiClient, role: Option<NodeRole>) -> Result<Vec<NodeState>> {
    let request = ListNodeStateRequest { role };
    let node_states = client.list_node_state(&request).await?;
    Ok(node_states)
}

pub async fn list_api_state(client: &ApiClient) -> Result<Vec<NodeState>> {
    list_node_state(client, Some(NodeRole::Api)).await
}

pub async fn list_builder_state(client: &ApiClient) -> Result<Vec<NodeState>> {
    list_node_state(client, Some(NodeRole::Builder)).await
}

pub async fn list_repository_state(client: &ApiClient) -> Result<Vec<NodeState>> {
    list_node_state(client, Some(NodeRole::Repository)).await
}

pub async fn list_scheduler_state(client: &ApiClient) -> Result<Vec<NodeState>> {
    list_node_state(client, Some(NodeRole::Scheduler)).await
}
