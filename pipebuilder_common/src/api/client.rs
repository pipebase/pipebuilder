use super::{
    constants::{
        ACTIVATE_NODE, APP, APP_METADATA, BUILD, BUILD_CACHE, BUILD_LOG, BUILD_METADATA,
        BUILD_SNAPSHOT, CANCEL_BUILD, CATALOGS, CATALOGS_METADATA, CATALOGS_SNAPSHOT,
        CATALOG_SCHEMA, CATALOG_SCHEMA_METADATA, CATALOG_SCHEMA_SNAPSHOT, DEACTIVATE_NODE,
        MANIFEST, MANIFEST_METADATA, MANIFEST_SNAPSHOT, NAMESPACE, NODE_STATE, PROJECT, SCAN_BUILD,
        SCAN_BUILD_CACHE, SHUTDOWN, SHUTDOWN_NODE,
    },
    models,
};
use crate::{
    api_client_error, api_server_error, Catalog, CatalogSchemaValidator, CatalogsNameValidator,
    Result, ValidateCatalog,
};
use reqwest::{
    header::{HeaderMap, HeaderName},
    Body, Client, Response,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;

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
    pub headers: Option<HashMap<String, String>>,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        ApiClientConfig {
            endpoint: String::from("http://127.0.0.1:16000"),
            basic_auth: None,
            bearer_auth_token: None,
            headers: None,
        }
    }
}

fn build_header_map(headers: &HashMap<String, String>) -> HeaderMap {
    let mut hmap = HeaderMap::new();
    for (name, value) in headers {
        hmap.insert::<HeaderName>(
            name.parse()
                .unwrap_or_else(|_| panic!("invalid header name '{}'", name)),
            value
                .parse()
                .unwrap_or_else(|_| panic!("invalid header value '{}'", value)),
        );
    }
    hmap
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
        let hmap: HeaderMap = match headers {
            Some(headers) => build_header_map(&headers),
            None => HeaderMap::new(),
        };
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

    pub async fn delete<B>(&self, path: &str, body: B) -> Result<Response>
    where
        B: Into<Body>,
    {
        let req = self
            .client
            .delete(self.get_url(path))
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

    pub async fn build(&self, request: &models::BuildRequest) -> Result<models::BuildResponse> {
        let request = Self::serialize_request(request)?;
        let response = self.post(BUILD, request).await?;
        let response = Self::get_response_body::<models::BuildResponse>(response).await?;
        Ok(response)
    }

    pub async fn get_build_metadata(
        &self,
        request: &models::GetBuildRequest,
    ) -> Result<models::BuildMetadata> {
        let response = self.query(BUILD_METADATA, request).await?;
        let response = Self::get_response_body::<models::BuildMetadata>(response).await?;
        Ok(response)
    }

    pub async fn list_build_metadata(
        &self,
        request: &models::ListBuildRequest,
    ) -> Result<Vec<models::BuildMetadata>> {
        let response = self.query(BUILD_METADATA, request).await?;
        let response = Self::get_response_body::<Vec<models::BuildMetadata>>(response).await?;
        Ok(response)
    }

    pub async fn cancel_build(
        &self,
        request: &models::CancelBuildRequest,
    ) -> Result<models::CancelBuildResponse> {
        let request = Self::serialize_request(request)?;
        let response = self.post(CANCEL_BUILD, request).await?;
        let response = Self::get_response_body::<models::CancelBuildResponse>(response).await?;
        Ok(response)
    }

    pub async fn pull_build_log(
        &self,
        request: &models::GetBuildLogRequest,
    ) -> Result<models::GetBuildLogResponse> {
        let response = self.query(BUILD_LOG, request).await?;
        let response = Self::get_response_body::<models::GetBuildLogResponse>(response).await?;
        Ok(response)
    }

    pub async fn delete_build(&self, request: &models::DeleteBuildRequest) -> Result<()> {
        let request = Self::serialize_request(request)?;
        let _ = self.delete(BUILD, request).await?;
        Ok(())
    }

    pub async fn scan_build(
        &self,
        request: &models::ScanBuildRequest,
    ) -> Result<Vec<models::BuildMetadataKey>> {
        let response = self.query(SCAN_BUILD, request).await?;
        let response = Self::get_response_body::<Vec<models::BuildMetadataKey>>(response).await?;
        Ok(response)
    }

    pub async fn scan_build_cache(
        &self,
        request: &models::ScanBuildCacheRequest,
    ) -> Result<Vec<models::BuildCacheMetadata>> {
        let response = self.query(SCAN_BUILD_CACHE, request).await?;
        let response = Self::get_response_body::<Vec<models::BuildCacheMetadata>>(response).await?;
        Ok(response)
    }

    pub async fn delete_build_cache(
        &self,
        request: &models::DeleteBuildCacheRequest,
    ) -> Result<()> {
        let request = Self::serialize_request(request)?;
        let _ = self.delete(BUILD_CACHE, request).await?;
        Ok(())
    }

    pub async fn list_build_snapshot(
        &self,
        request: &models::ListBuildSnapshotRequest,
    ) -> Result<Vec<models::BuildSnapshot>> {
        let response = self.query(BUILD_SNAPSHOT, request).await?;
        let response = Self::get_response_body::<Vec<models::BuildSnapshot>>(response).await?;
        Ok(response)
    }

    pub async fn delete_build_snapshot(
        &self,
        request: &models::DeleteBuildSnapshotRequest,
    ) -> Result<()> {
        let request = Self::serialize_request(request)?;
        let _ = self.delete(BUILD_SNAPSHOT, request).await?;
        Ok(())
    }

    pub async fn pull_manifest(
        &self,
        request: &models::GetManifestRequest,
    ) -> Result<models::GetManifestResponse> {
        let response = self.query(MANIFEST, request).await?;
        let response = Self::get_response_body::<models::GetManifestResponse>(response).await?;
        Ok(response)
    }

    pub async fn push_manifest(
        &self,
        request: &models::PostManifestRequest,
    ) -> Result<models::PostManifestResponse> {
        let request = Self::serialize_request(request)?;
        let response = self.post(MANIFEST, request).await?;
        let response = Self::get_response_body::<models::PostManifestResponse>(response).await?;
        Ok(response)
    }

    pub async fn list_manifest_snapshot(
        &self,
        request: &models::ListManifestSnapshotRequest,
    ) -> Result<Vec<models::ManifestSnapshot>> {
        let response = self.query(MANIFEST_SNAPSHOT, request).await?;
        let response = Self::get_response_body::<Vec<models::ManifestSnapshot>>(response).await?;
        Ok(response)
    }

    pub async fn list_manifest_metadata(
        &self,
        request: &models::ListManifestMetadataRequest,
    ) -> Result<Vec<models::ManifestMetadata>> {
        let response = self.query(MANIFEST_METADATA, request).await?;
        let response = Self::get_response_body::<Vec<models::ManifestMetadata>>(response).await?;
        Ok(response)
    }

    pub async fn delete_manfiest(&self, request: &models::DeleteManifestRequest) -> Result<()> {
        let request = Self::serialize_request(request)?;
        let _ = self.delete(MANIFEST, request).await?;
        Ok(())
    }

    pub async fn delete_manifest_snapshot(
        &self,
        request: &models::DeleteManifestSnapshotRequest,
    ) -> Result<()> {
        let request = Self::serialize_request(request)?;
        let _ = self.delete(MANIFEST_SNAPSHOT, request).await?;
        Ok(())
    }

    pub async fn pull_catalogs(
        &self,
        request: &models::GetCatalogsRequest,
    ) -> Result<models::GetCatalogsResponse> {
        let response = self.query(CATALOGS, request).await?;
        let response = Self::get_response_body::<models::GetCatalogsResponse>(response).await?;
        Ok(response)
    }

    pub async fn push_catalogs(
        &self,
        request: &models::PostCatalogsRequest,
    ) -> Result<models::PostCatalogsResponse> {
        let request = Self::serialize_request(request)?;
        let response = self.post(CATALOGS, request).await?;
        let response = Self::get_response_body::<models::PostCatalogsResponse>(response).await?;
        Ok(response)
    }

    pub async fn list_catalogs_snapshot(
        &self,
        request: &models::ListCatalogsSnapshotRequest,
    ) -> Result<Vec<models::CatalogsSnapshot>> {
        let response = self.query(CATALOGS_SNAPSHOT, request).await?;
        let response = Self::get_response_body::<Vec<models::CatalogsSnapshot>>(response).await?;
        Ok(response)
    }

    pub async fn list_catalogs_metadata(
        &self,
        request: &models::ListCatalogsMetadataRequest,
    ) -> Result<Vec<models::CatalogsMetadata>> {
        let response = self.query(CATALOGS_METADATA, request).await?;
        let response = Self::get_response_body::<Vec<models::CatalogsMetadata>>(response).await?;
        Ok(response)
    }

    pub async fn delete_catalogs(&self, request: &models::DeleteCatalogsRequest) -> Result<()> {
        let request = Self::serialize_request(request)?;
        let _ = self.delete(CATALOGS, request).await?;
        Ok(())
    }

    pub async fn delete_catalogs_snapshot(
        &self,
        request: &models::DeleteCatalogsSnapshotRequest,
    ) -> Result<()> {
        let request = Self::serialize_request(request)?;
        let _ = self.delete(CATALOGS_SNAPSHOT, request).await?;
        Ok(())
    }

    pub async fn pull_catalog_schema(
        &self,
        request: &models::GetCatalogSchemaRequest,
    ) -> Result<models::GetCatalogSchemaResponse> {
        let response = self.query(CATALOG_SCHEMA, request).await?;
        let response =
            Self::get_response_body::<models::GetCatalogSchemaResponse>(response).await?;
        Ok(response)
    }

    pub async fn push_catalog_schema(
        &self,
        request: &models::PostCatalogSchemaRequest,
    ) -> Result<models::PostCatalogSchemaResponse> {
        let request = Self::serialize_request(request)?;
        let response = self.post(CATALOG_SCHEMA, request).await?;
        let response =
            Self::get_response_body::<models::PostCatalogSchemaResponse>(response).await?;
        Ok(response)
    }

    pub async fn list_catalog_schema_snapshot(
        &self,
        request: &models::ListCatalogSchemaSnapshotRequest,
    ) -> Result<Vec<models::CatalogSchemaSnapshot>> {
        let response = self.query(CATALOG_SCHEMA_SNAPSHOT, request).await?;
        let response =
            Self::get_response_body::<Vec<models::CatalogSchemaSnapshot>>(response).await?;
        Ok(response)
    }

    pub async fn list_catalog_schema_metadata(
        &self,
        request: &models::ListCatalogSchemaMetadataRequest,
    ) -> Result<Vec<models::CatalogSchemaMetadata>> {
        let response = self.query(CATALOG_SCHEMA_METADATA, request).await?;
        let response =
            Self::get_response_body::<Vec<models::CatalogSchemaMetadata>>(response).await?;
        Ok(response)
    }

    pub async fn delete_catalog_schema(
        &self,
        request: &models::DeleteCatalogSchemaRequest,
    ) -> Result<()> {
        let request = Self::serialize_request(request)?;
        let _ = self.delete(CATALOG_SCHEMA, request).await?;
        Ok(())
    }

    pub async fn delete_catalog_schema_snapshot(
        &self,
        request: &models::DeleteCatalogSchemaSnapshotRequest,
    ) -> Result<()> {
        let request = Self::serialize_request(request)?;
        let _ = self.delete(CATALOG_SCHEMA_SNAPSHOT, request).await?;
        Ok(())
    }

    pub async fn pull_app(
        &self,
        request: &models::GetAppRequest,
    ) -> Result<models::GetAppResponse> {
        let response = self.query(APP, request).await?;
        let response = Self::get_response_body::<models::GetAppResponse>(response).await?;
        Ok(response)
    }

    pub async fn list_app_metadata(
        &self,
        request: &models::ListAppMetadataRequest,
    ) -> Result<Vec<models::AppMetadata>> {
        let response = self.query(APP_METADATA, request).await?;
        let response = Self::get_response_body::<Vec<models::AppMetadata>>(response).await?;
        Ok(response)
    }

    pub async fn delete_app(&self, request: &models::DeleteAppRequest) -> Result<()> {
        let request = Self::serialize_request(request)?;
        let _ = self.delete(APP, request).await?;
        Ok(())
    }

    pub async fn update_project(
        &self,
        request: &models::UpdateProjectRequest,
    ) -> Result<models::Project> {
        let request = Self::serialize_request(request)?;
        let response = self.post(PROJECT, request).await?;
        let response = Self::get_response_body::<models::Project>(response).await?;
        Ok(response)
    }

    pub async fn delete_project(&self, request: &models::DeleteProjectRequest) -> Result<()> {
        let request = Self::serialize_request(request)?;
        let _ = self.delete(PROJECT, request).await?;
        Ok(())
    }

    pub async fn list_project(
        &self,
        request: &models::ListProjectRequest,
    ) -> Result<Vec<models::Project>> {
        let response = self.query(PROJECT, request).await?;
        let response = Self::get_response_body::<Vec<models::Project>>(response).await?;
        Ok(response)
    }

    pub async fn update_namespace(
        &self,
        request: &models::UpdateNamespaceRequest,
    ) -> Result<models::Namespace> {
        let request = Self::serialize_request(request)?;
        let response = self.post(NAMESPACE, request).await?;
        let response = Self::get_response_body::<models::Namespace>(response).await?;
        Ok(response)
    }

    pub async fn list_namespace(
        &self,
        request: &models::ListNamespaceRequest,
    ) -> Result<Vec<models::Namespace>> {
        let response = self.query(NAMESPACE, request).await?;
        let response = Self::get_response_body::<Vec<models::Namespace>>(response).await?;
        Ok(response)
    }

    pub async fn delete_namespace(&self, request: &models::DeleteNamespaceRequest) -> Result<()> {
        let request = Self::serialize_request(request)?;
        let _ = self.delete(NAMESPACE, request).await?;
        Ok(())
    }

    pub async fn list_node_state(
        &self,
        request: &models::ListNodeStateRequest,
    ) -> Result<Vec<models::NodeState>> {
        let response = self.query(NODE_STATE, request).await?;
        let response = Self::get_response_body::<Vec<models::NodeState>>(response).await?;
        Ok(response)
    }

    pub async fn activate_node(
        &self,
        request: &models::ActivateNodeRequest,
    ) -> Result<models::ActivateNodeResponse> {
        let request = Self::serialize_request(request)?;
        let response = self.post(ACTIVATE_NODE, request).await?;
        let response = Self::get_response_body::<models::ActivateNodeResponse>(response).await?;
        Ok(response)
    }

    pub async fn deactivate_node(
        &self,
        request: &models::DeactivateNodeRequest,
    ) -> Result<models::DeactivateNodeResponse> {
        let request = Self::serialize_request(request)?;
        let response = self.post(DEACTIVATE_NODE, request).await?;
        let response = Self::get_response_body::<models::DeactivateNodeResponse>(response).await?;
        Ok(response)
    }

    // shutdown internal node except api
    pub async fn shutdown_node(
        &self,
        request: &models::ShutdownNodeRequest,
    ) -> Result<models::ShutdownNodeResponse> {
        let request = Self::serialize_request(request)?;
        let response = self.post(SHUTDOWN_NODE, request).await?;
        let response = Self::get_response_body::<models::ShutdownNodeResponse>(response).await?;
        Ok(response)
    }

    // shutdown api
    pub async fn shutdown(
        &self,
        request: &models::ShutdownRequest,
    ) -> Result<models::ShutdownResponse> {
        let request = Self::serialize_request(request)?;
        let response = self.post(SHUTDOWN, request).await?;
        let response = Self::get_response_body::<models::ShutdownResponse>(response).await?;
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
        let message = match serde_json::from_slice::<models::Failure>(&buffer) {
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

    pub fn validate_manifest(manifest: &[u8]) -> Result<()> {
        let app = pipegen::models::App::read_from_buffer(manifest)?;
        app.validate()?;
        Ok(())
    }

    pub fn validate_catalog_schema(schema: &[u8]) -> Result<()> {
        let _ = CatalogSchemaValidator::from_buffer(schema)?;
        Ok(())
    }

    pub async fn validate_catalogs(&self, catalogs: &[u8]) -> Result<()> {
        // deserialize catalogs
        let catalogs = Catalog::from_buffer(catalogs)?;
        // validate catalog naming
        Self::validate_catalogs_name(catalogs.as_slice())?;
        // validate catalog against catalog schema
        for catalog in catalogs.iter() {
            self.validate_catalog(catalog).await?;
        }
        Ok(())
    }

    fn validate_catalogs_name(catalogs: &[Catalog]) -> Result<()> {
        let mut validator = CatalogsNameValidator::default();
        for catalog in catalogs {
            catalog.accept(&mut validator)?;
        }
        validator.validate()
    }

    async fn validate_catalog(&self, catalog: &Catalog) -> Result<()> {
        let schema = catalog.get_schema_metadata_key();
        let namespace = schema.namespace.to_owned();
        let id = schema.id.to_owned();
        let version = schema.version;
        // TODO: Catalog schema caching at client side
        let request = models::GetCatalogSchemaRequest {
            namespace,
            id,
            version,
        };
        let resp = self.pull_catalog_schema(&request).await?;
        let buffer = resp.buffer;
        let mut validator = CatalogSchemaValidator::from_buffer(buffer.as_slice())?;
        catalog.accept(&mut validator)?;
        validator.validate()
    }

    pub async fn dump_catalogs<P>(catalogs: &[u8], directory: P) -> Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        Catalog::dump_catalogs(catalogs, directory).await
    }
}
