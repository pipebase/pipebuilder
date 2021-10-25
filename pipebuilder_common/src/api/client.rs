use super::{
    constants::{
        ACTIVATE_NODE, APP, APP_METADATA, BUILD, BUILD_LOG, BUILD_SNAPSHOT, CANCEL_BUILD,
        DEACTIVATE_NODE, MANIFEST, MANIFEST_METADATA, MANIFEST_SNAPSHOT, NAMESPACE, NODE_STATE,
        PROJECT, SCAN_BUILDER,
    },
    models::{
        ActivateNodeRequest, ActivateNodeResponse, AppMetadata, BuildRequest, BuildResponse,
        BuildSnapshot, CancelBuildRequest, CancelBuildResponse, DeactivateNodeRequest,
        DeactivateNodeResponse, Failure, GetAppRequest, GetAppResponse, GetBuildLogRequest,
        GetBuildLogResponse, GetBuildRequest, GetManifestRequest, GetManifestResponse,
        ListAppMetadataRequest, ListBuildRequest, ListBuildSnapshotRequest,
        ListManifestMetadataRequest, ListManifestSnapshotRequest, ListNamespaceRequest,
        ListNodeStateRequest, ListProjectRequest, ManifestMetadata, ManifestSnapshot, Namespace,
        NodeState, Project, PutManifestRequest, PutManifestResponse, ScanBuilderRequest,
        UpdateNamespaceRequest, UpdateProjectRequest, VersionBuild, VersionBuildKey,
    },
};
use crate::{api_client_error, api_server_error, Result};
use reqwest::{
    header::{HeaderMap, HeaderName},
    Body, Client, Response,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Clone, Deserialize)]
pub struct BasicAuth {
    pub username: String,
    pub password: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct ApiClientConfig {
    // api endpoint
    pub endpoint: String,
    pub basic_auth: Option<BasicAuth>,
    pub bearer_auth_token: Option<String>,
    pub headers: HashMap<String, String>,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        ApiClientConfig {
            endpoint: String::from("http://127.0.0.1:16000"),
            basic_auth: None,
            bearer_auth_token: None,
            headers: HashMap::new(),
        }
    }
}

pub struct ApiClient {
    client: Client,
    endpoint: String,
    basic_auth: Option<BasicAuth>,
    bearer_auth_token: Option<String>,
    headers: HeaderMap,
}

impl From<ApiClientConfig> for ApiClient {
    fn from(config: ApiClientConfig) -> Self {
        let endpoint = config.endpoint;
        let basic_auth = config.basic_auth;
        let bearer_auth_token = config.bearer_auth_token;
        let headers = config.headers;
        let mut hmap = HeaderMap::new();
        for (name, value) in &headers {
            hmap.insert::<HeaderName>(
                name.parse()
                    .unwrap_or_else(|_| panic!("invalid header name '{}'", name)),
                value
                    .parse()
                    .unwrap_or_else(|_| panic!("invalid header value '{}'", value)),
            );
        }
        ApiClient {
            client: Client::new(),
            endpoint,
            basic_auth,
            bearer_auth_token,
            headers: hmap,
        }
    }
}

impl ApiClient {
    fn get_url(&self, path: &str) -> String {
        format!("{}{}", self.endpoint, path)
    }

    pub async fn post<B>(&self, path: &str, body: B) -> Result<Response>
    where
        B: Into<Body>,
    {
        let req = self
            .client
            .post(self.get_url(path))
            .headers(self.headers.to_owned());
        let req = match self.basic_auth {
            Some(ref basic_auth) => {
                req.basic_auth(&basic_auth.username, basic_auth.password.as_ref())
            }
            None => req,
        };
        let req = match self.bearer_auth_token {
            Some(ref token) => req.bearer_auth(token),
            None => req,
        };
        let resp = req.body(body).send().await?;
        Ok(resp)
    }

    pub async fn query<Q>(&self, path: &str, query: &Q) -> Result<Response>
    where
        Q: Serialize,
    {
        let req = self
            .client
            .get(self.get_url(path))
            .headers(self.headers.to_owned())
            .query(query);
        let req = match self.basic_auth {
            Some(ref basic_auth) => {
                req.basic_auth(&basic_auth.username, basic_auth.password.as_ref())
            }
            None => req,
        };
        let req = match self.bearer_auth_token {
            Some(ref token) => req.bearer_auth(token),
            None => req,
        };
        let resp = req.send().await?;
        Ok(resp)
    }

    pub async fn build(&self, request: &BuildRequest) -> Result<BuildResponse> {
        let request = Self::serialize_request(request)?;
        let response = self.post(BUILD, request).await?;
        let response = Self::get_response_body::<BuildResponse>(response).await?;
        Ok(response)
    }

    pub async fn get_build(&self, request: &GetBuildRequest) -> Result<VersionBuild> {
        let response = self.query(BUILD, request).await?;
        let response = Self::get_response_body::<VersionBuild>(response).await?;
        Ok(response)
    }

    pub async fn list_build(&self, request: &ListBuildRequest) -> Result<Vec<VersionBuild>> {
        let response = self.query(BUILD, request).await?;
        let response = Self::get_response_body::<Vec<VersionBuild>>(response).await?;
        Ok(response)
    }

    pub async fn cancel_build(&self, request: &CancelBuildRequest) -> Result<CancelBuildResponse> {
        let request = Self::serialize_request(request)?;
        let response = self.post(CANCEL_BUILD, request).await?;
        let response = Self::get_response_body::<CancelBuildResponse>(response).await?;
        Ok(response)
    }

    pub async fn list_build_snapshot(
        &self,
        request: &ListBuildSnapshotRequest,
    ) -> Result<Vec<BuildSnapshot>> {
        let response = self.query(BUILD_SNAPSHOT, request).await?;
        let response = Self::get_response_body::<Vec<BuildSnapshot>>(response).await?;
        Ok(response)
    }

    pub async fn get_manifest(&self, request: &GetManifestRequest) -> Result<GetManifestResponse> {
        let response = self.query(MANIFEST, request).await?;
        let response = Self::get_response_body::<GetManifestResponse>(response).await?;
        Ok(response)
    }

    pub async fn put_manifest(&self, request: &PutManifestRequest) -> Result<PutManifestResponse> {
        let request = Self::serialize_request(request)?;
        let response = self.post(MANIFEST, request).await?;
        let response = Self::get_response_body::<PutManifestResponse>(response).await?;
        Ok(response)
    }

    pub async fn list_manifest_snapshot(
        &self,
        request: &ListManifestSnapshotRequest,
    ) -> Result<Vec<ManifestSnapshot>> {
        let response = self.query(MANIFEST_SNAPSHOT, request).await?;
        let response = Self::get_response_body::<Vec<ManifestSnapshot>>(response).await?;
        Ok(response)
    }

    pub async fn get_app(&self, request: &GetAppRequest) -> Result<GetAppResponse> {
        let response = self.query(APP, request).await?;
        let response = Self::get_response_body::<GetAppResponse>(response).await?;
        Ok(response)
    }

    pub async fn get_build_log(&self, request: &GetBuildLogRequest) -> Result<GetBuildLogResponse> {
        let response = self.query(BUILD_LOG, request).await?;
        let response = Self::get_response_body::<GetBuildLogResponse>(response).await?;
        Ok(response)
    }

    pub async fn list_node_state(&self, request: &ListNodeStateRequest) -> Result<Vec<NodeState>> {
        let response = self.query(NODE_STATE, request).await?;
        let response = Self::get_response_body::<Vec<NodeState>>(response).await?;
        Ok(response)
    }

    pub async fn scan_builder(&self, request: &ScanBuilderRequest) -> Result<Vec<VersionBuildKey>> {
        let response = self.query(SCAN_BUILDER, request).await?;
        let response = Self::get_response_body::<Vec<VersionBuildKey>>(response).await?;
        Ok(response)
    }

    pub async fn activate_node(
        &self,
        request: &ActivateNodeRequest,
    ) -> Result<ActivateNodeResponse> {
        let request = Self::serialize_request(request)?;
        let response = self.post(ACTIVATE_NODE, request).await?;
        let response = Self::get_response_body::<ActivateNodeResponse>(response).await?;
        Ok(response)
    }

    pub async fn deactivate_node(
        &self,
        request: &DeactivateNodeRequest,
    ) -> Result<DeactivateNodeResponse> {
        let request = Self::serialize_request(request)?;
        let response = self.post(DEACTIVATE_NODE, request).await?;
        let response = Self::get_response_body::<DeactivateNodeResponse>(response).await?;
        Ok(response)
    }

    pub async fn list_app_metadata(
        &self,
        request: &ListAppMetadataRequest,
    ) -> Result<Vec<AppMetadata>> {
        let response = self.query(APP_METADATA, request).await?;
        let response = Self::get_response_body::<Vec<AppMetadata>>(response).await?;
        Ok(response)
    }

    pub async fn list_manifest_metadata(
        &self,
        request: &ListManifestMetadataRequest,
    ) -> Result<Vec<ManifestMetadata>> {
        let response = self.query(MANIFEST_METADATA, request).await?;
        let response = Self::get_response_body::<Vec<ManifestMetadata>>(response).await?;
        Ok(response)
    }

    pub async fn update_namespace(&self, request: &UpdateNamespaceRequest) -> Result<Namespace> {
        let request = Self::serialize_request(request)?;
        let response = self.post(NAMESPACE, request).await?;
        let response = Self::get_response_body::<Namespace>(response).await?;
        Ok(response)
    }

    pub async fn update_project(&self, request: &UpdateProjectRequest) -> Result<Project> {
        let request = Self::serialize_request(request)?;
        let response = self.post(PROJECT, request).await?;
        let response = Self::get_response_body::<Project>(response).await?;
        Ok(response)
    }

    pub async fn list_namespace(&self, request: &ListNamespaceRequest) -> Result<Vec<Namespace>> {
        let response = self.query(NAMESPACE, request).await?;
        let response = Self::get_response_body::<Vec<Namespace>>(response).await?;
        Ok(response)
    }

    pub async fn list_project(&self, request: &ListProjectRequest) -> Result<Vec<Project>> {
        let response = self.query(PROJECT, request).await?;
        let response = Self::get_response_body::<Vec<Project>>(response).await?;
        Ok(response)
    }

    fn serialize_request<T>(request: &T) -> Result<Vec<u8>>
    where
        T: Serialize,
    {
        let buffer = serde_json::to_vec(request)?;
        Ok(buffer)
    }

    async fn get_response_body<T>(response: Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        if status.is_success() {
            let buffer = response.bytes().await?;
            let buffer = buffer.to_vec();
            let t = serde_json::from_slice::<T>(&buffer)?;
            return Ok(t);
        }
        let status_code = status.as_u16();
        let reason = status.canonical_reason().map(String::from);
        // parse failure message
        let buffer = response.bytes().await?;
        let buffer = buffer.to_vec();
        let message = match serde_json::from_slice::<Failure>(&buffer) {
            Ok(failure) => Some(failure.error),
            Err(_) => None,
        };
        if status.is_client_error() {
            return Err(api_client_error(status_code, reason, message));
        }
        if status.is_server_error() {
            return Err(api_server_error(status_code, reason, message));
        }
        unreachable!()
    }

    pub fn validate_manifest(&self, path: &str) -> Result<()> {
        let path = PathBuf::from(path);
        let app = pipegen::models::App::read_from_path(path.as_path())?;
        app.validate()?;
        Ok(())
    }
}
