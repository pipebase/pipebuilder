use std::path::Path;

use pipebuilder_common::{
    api::{
        client::{ApiClient, ApiClientConfig},
        models::{
            BuildCacheMetadata, BuildMetadata, BuildRequest, BuildResponse,
            DeleteBuildCacheRequest, GetBuildRequest, ListManifestMetadataRequest,
            ListNamespaceRequest, ListNodeStateRequest, ListProjectRequest, ManifestMetadata,
            Namespace, NodeState, PostManifestRequest, PostManifestResponse, Project,
            ScanBuildCacheRequest, ShutdownNodeRequest, ShutdownRequest, UpdateNamespaceRequest,
            UpdateProjectRequest,
        },
    },
    open_file, parse_config, NodeRole, Result,
};

use tokio::time::{sleep, Duration};

// api client

pub async fn build_api_client<P>(path: P) -> Result<ApiClient>
where
    P: AsRef<Path>,
{
    let config_file = open_file(path).await?;
    let config = parse_config::<ApiClientConfig>(config_file).await?;
    Ok(config.into())
}

// node

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

// namespace

pub async fn create_namespace(client: &ApiClient, id: String) -> Result<Namespace> {
    let request = UpdateNamespaceRequest { id };
    let namespace = client.update_namespace(&request).await?;
    Ok(namespace)
}

pub async fn list_namespace(client: &ApiClient) -> Result<Vec<Namespace>> {
    let request = ListNamespaceRequest {};
    let namespaces = client.list_namespace(&request).await?;
    Ok(namespaces)
}

// project
pub async fn create_project(client: &ApiClient, namespace: String, id: String) -> Result<Project> {
    let request = UpdateProjectRequest { namespace, id };
    let project = client.update_project(&request).await?;
    Ok(project)
}

pub async fn list_project(client: &ApiClient, namespace: String) -> Result<Vec<Project>> {
    let request = ListProjectRequest { namespace };
    let projects = client.list_project(&request).await?;
    Ok(projects)
}

// manifest
pub async fn push_manifest(
    client: &ApiClient,
    namespace: String,
    id: String,
    buffer: Vec<u8>,
) -> Result<PostManifestResponse> {
    let request = PostManifestRequest {
        namespace,
        id,
        buffer,
    };
    client.push_manifest(&request).await
}

pub async fn list_manifest_metadata(
    client: &ApiClient,
    namespace: String,
    id: Option<String>,
) -> Result<Vec<ManifestMetadata>> {
    let request = ListManifestMetadataRequest { namespace, id };
    client.list_manifest_metadata(&request).await
}

// build
pub async fn build(
    client: &ApiClient,
    namespace: String,
    id: String,
    manifest_version: u64,
    target_platform: Option<String>,
) -> Result<BuildResponse> {
    let request = BuildRequest {
        namespace,
        id,
        manifest_version,
        target_platform,
    };
    client.build(&request).await
}

pub async fn get_build_metadata(
    client: &ApiClient,
    namespace: String,
    id: String,
    version: u64,
) -> Result<BuildMetadata> {
    let request = GetBuildRequest {
        namespace,
        id,
        version,
    };
    client.get_build_metadata(&request).await
}

pub async fn scan_build_cache_metadata(
    client: &ApiClient,
    builder_id: String,
) -> Result<Vec<BuildCacheMetadata>> {
    let request = ScanBuildCacheRequest { builder_id };
    client.scan_build_cache(&request).await
}

pub async fn delete_build_cache(
    client: &ApiClient,
    builder_id: String,
    namespace: String,
    id: String,
    target_platform: String,
) -> Result<()> {
    let request = DeleteBuildCacheRequest {
        builder_id,
        namespace,
        id,
        target_platform,
    };
    client.delete_build_cache(&request).await
}

pub async fn wait(millis: u64) {
    sleep(Duration::from_millis(millis)).await;
}
