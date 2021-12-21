use pipebuilder_common::{
    api::models, invalid_api_request, AppMetadata, Build, BuildMetadata, BuildSnapshot,
    ManifestMetadata, ManifestSnapshot, NodeRole, NodeState, Project, Register, ResourceKeyBuilder,
    ResourceType, Result,
};

pub async fn validate_build_request(
    register: &mut Register,
    request: &models::BuildRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = request.id.as_str();
    validate_project(register, namespace, id).await?;
    if let Some(target_platform) = request.target_platform.as_ref() {
        validate_target_platform(target_platform)?;
    };
    Ok(())
}

pub async fn validate_get_build_request(
    register: &mut Register,
    request: &models::GetBuildRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = request.id.as_str();
    validate_project(register, namespace, id).await
}

pub async fn validate_get_build_log_request(
    register: &mut Register,
    request: &models::GetBuildLogRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = request.id.as_str();
    validate_project(register, namespace, id).await
}

pub async fn validate_cancel_build_request(
    register: &mut Register,
    request: &models::CancelBuildRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = request.id.as_str();
    validate_project(register, namespace, id).await
}

pub async fn validate_delete_build_request(
    register: &mut Register,
    request: &models::DeleteBuildRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = request.id.as_str();
    validate_project(register, namespace, id).await
}

pub async fn validate_list_build_request(
    register: &mut Register,
    request: &models::ListBuildRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = match request.id.as_ref() {
        Some(id) => id,
        None => return Ok(()),
    };
    validate_project(register, namespace, id).await
}

pub async fn validate_list_build_snapshot_request(
    register: &mut Register,
    request: &models::ListBuildSnapshotRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await
}

pub async fn validate_post_manifest_request(
    register: &mut Register,
    request: &models::PostManifestRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = request.id.as_str();
    validate_project(register, namespace, id).await
}

pub async fn validate_get_manifest_request(
    register: &mut Register,
    request: &models::GetManifestRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = request.id.as_str();
    validate_project(register, namespace, id).await
}

pub async fn validate_delete_manifest_request(
    register: &mut Register,
    request: &models::DeleteManifestRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = request.id.as_str();
    validate_project(register, namespace, id).await
}

pub async fn validate_list_manifest_snapshot_request(
    register: &mut Register,
    request: &models::ListManifestSnapshotRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await
}

pub async fn validate_list_manifest_metadata_request(
    register: &mut Register,
    request: &models::ListManifestMetadataRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = match request.id.as_ref() {
        Some(id) => id,
        None => return Ok(()),
    };
    validate_project(register, namespace, id).await
}

pub async fn validate_list_app_metadata_request(
    register: &mut Register,
    request: &models::ListAppMetadataRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = match request.id.as_ref() {
        Some(id) => id,
        None => return Ok(()),
    };
    validate_project(register, namespace, id).await
}

pub async fn validate_get_app_request(
    register: &mut Register,
    request: &models::GetAppRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = request.id.as_str();
    validate_project(register, namespace, id).await
}

pub async fn validate_delete_app_request(
    register: &mut Register,
    request: &models::DeleteAppRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = request.id.as_str();
    validate_project(register, namespace, id).await
}

pub async fn validate_list_project_request(
    register: &mut Register,
    request: &models::ListProjectRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await
}

pub async fn validate_put_project_request(
    register: &mut Register,
    request: &models::UpdateProjectRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await
}

pub async fn validate_delete_manifest_snapshot_request(
    register: &mut Register,
    request: &models::DeleteManifestSnapshotRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = request.id.as_str();
    validate_project(register, namespace, id).await?;
    match is_manifest_metadata_exist(register, namespace, id).await? {
        true => Err(invalid_api_request(format!(
            "can not delete manifest snapshot (namespace = {}, id = {}), manifests found",
            namespace, id
        ))),
        false => Ok(()),
    }
}

pub async fn validate_delete_build_snapshot_request(
    register: &mut Register,
    request: &models::DeleteBuildSnapshotRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = request.id.as_str();
    validate_project(register, namespace, id).await?;
    match is_build_metadata_exist(register, namespace, id).await? {
        true => Err(invalid_api_request(format!(
            "can not delete build snapshot (namespace = {}, id = {}), builds found",
            namespace, id
        ))),
        false => Ok(()),
    }
}

pub async fn validate_delete_project_request(
    register: &mut Register,
    request: &models::DeleteProjectRequest,
) -> Result<()> {
    let namespace = request.namespace.as_str();
    validate_namespace(register, namespace).await?;
    let id = request.id.as_str();
    validate_project(register, namespace, id).await?;
    match is_build_snapshot_exist(register, namespace, id).await? {
        true => {
            return Err(invalid_api_request(format!(
                "can not delete project (namespace = {}, id = {}), build snapshot found.",
                namespace, id
            )))
        }
        false => (),
    };
    match is_manifest_snapshot_exist(register, namespace, id).await? {
        true => {
            return Err(invalid_api_request(format!(
                "can not delete project (namespace = {}, id = {}), manifest snapshot found.",
                namespace, id
            )))
        }
        false => (),
    };
    match is_app_metadata_exist(register, namespace, id).await? {
        true => Err(invalid_api_request(format!(
            "can not delete project (namespace = {}, id = {}), app metadata found.",
            namespace, id
        ))),
        false => Ok(()),
    }
}

pub async fn validate_delete_namespace_request(
    register: &mut Register,
    request: &models::DeleteNamespaceRequest,
) -> Result<()> {
    let id = request.id.as_str();
    match is_project_exist(register, id).await? {
        true => Err(invalid_api_request(format!(
            "can not delete namespace '{}', project found",
            id
        ))),
        false => Ok(()),
    }
}

pub fn validate_list_node_state_request(request: &models::ListNodeStateRequest) -> Result<()> {
    let role = request.role.as_ref();
    let role = match role {
        Some(role) => role,
        None => return Ok(()),
    };
    validate_role(role)
}

fn validate_role(role: &NodeRole) -> Result<()> {
    match role {
        NodeRole::Undefined => Err(invalid_api_request(String::from("undefined node role"))),
        _ => Ok(()),
    }
}

pub fn validate_node_state(state: &NodeState, expected_role: &NodeRole) -> Result<()> {
    let actual_role = &state.role;
    if actual_role != expected_role {
        return Err(invalid_api_request(format!(
            "invalid node state, expect '{}', actual '{}'",
            expected_role.to_string(),
            actual_role.to_string()
        )));
    }
    Ok(())
}

fn validate_target_platform(target_platform: &str) -> Result<()> {
    if !Build::is_target_platform_support(target_platform) {
        return Err(invalid_api_request(format!(
            "target platform '{}' not support",
            target_platform
        )));
    }
    Ok(())
}

async fn validate_namespace(register: &mut Register, namespace: &str) -> Result<()> {
    let key = ResourceKeyBuilder::new()
        .resource(ResourceType::Namespace)
        .id(namespace)
        .build();
    let is_exist = register.is_exist(key).await?;
    match is_exist {
        true => Ok(()),
        false => Err(invalid_api_request(format!(
            "invalid namespace '{}'",
            namespace
        ))),
    }
}

async fn validate_project(register: &mut Register, namespace: &str, id: &str) -> Result<()> {
    let key = ResourceKeyBuilder::new()
        .resource(ResourceType::Project)
        .namespace(namespace)
        .id(id)
        .build();
    let is_exist = register.is_exist(key).await?;
    match is_exist {
        true => Ok(()),
        false => Err(invalid_api_request(format!(
            "invalid project (namespace = {}, id = {})",
            namespace, id
        ))),
    }
}

async fn is_manifest_metadata_exist(
    register: &mut Register,
    namespace: &str,
    id: &str,
) -> Result<bool> {
    register
        .is_resource_exist::<ManifestMetadata>(namespace, Some(id))
        .await
}

// check build metadata exists or not
async fn is_build_metadata_exist(
    register: &mut Register,
    namespace: &str,
    id: &str,
) -> Result<bool> {
    register
        .is_resource_exist::<BuildMetadata>(namespace, Some(id))
        .await
}

async fn is_build_snapshot_exist(
    register: &mut Register,
    namespace: &str,
    id: &str,
) -> Result<bool> {
    register
        .is_resource_exist::<BuildSnapshot>(namespace, Some(id))
        .await
}

async fn is_manifest_snapshot_exist(
    register: &mut Register,
    namespace: &str,
    id: &str,
) -> Result<bool> {
    register
        .is_resource_exist::<ManifestSnapshot>(namespace, Some(id))
        .await
}

async fn is_app_metadata_exist(register: &mut Register, namespace: &str, id: &str) -> Result<bool> {
    register
        .is_resource_exist::<AppMetadata>(namespace, Some(id))
        .await
}

async fn is_project_exist(register: &mut Register, namespace: &str) -> Result<bool> {
    register.is_resource_exist::<Project>(namespace, None).await
}
