use pipebuilder_common::{
    self, create_directory,
    grpc::repository::{
        repository_server::Repository, DeleteAppResponse, DeleteCatalogSchemaResponse,
        DeleteCatalogsResponse, DeleteManifestResponse, GetAppResponse, GetCatalogSchemaResponse,
        GetCatalogsResponse, GetManifestResponse, PostAppResponse, PutCatalogSchemaResponse,
        PutCatalogsResponse, PutManifestResponse,
    },
    read_file, repository_error, reset_directory, rpc_internal_error, write_file, AppMetadata,
    BlobDescriptor, BlobResource, CatalogSchemaMetadata, CatalogSchemaSnapshot, CatalogsMetadata,
    CatalogsSnapshot, ManifestMetadata, ManifestSnapshot, PathBuilder, Register, Resource,
    Snapshot, SnapshotDescriptor,
};
use serde::{de::DeserializeOwned, Serialize};
use std::fs::remove_dir_all;
use tonic::Response;
use tracing::{error, info};

pub const TARGET_MANIFEST: &str = "pipe.yml";
pub const TARGET_APP: &str = "app";
pub const TARGET_CATALOG_SCHEMA: &str = "schema.json";
pub const TARGET_CATALOGS: &str = "catalogs.yml";

#[derive(Default)]
pub struct RepositoryManagerBuilder {
    register: Option<Register>,
    lease_id: Option<i64>,
    // app binary directory
    app_directory: Option<String>,
    // manifest file directory
    manifest_directory: Option<String>,
    // catalog schema directory
    catalog_schema_directory: Option<String>,
    // catalogs directory
    catalogs_directory: Option<String>,
}

impl RepositoryManagerBuilder {
    pub fn register(mut self, register: Register) -> Self {
        self.register = Some(register);
        self
    }

    pub fn lease_id(mut self, lease_id: i64) -> Self {
        self.lease_id = Some(lease_id);
        self
    }

    pub fn app_directory(mut self, app_directory: String) -> Self {
        self.app_directory = Some(app_directory);
        self
    }

    pub fn manifest_directory(mut self, manifest_directory: String) -> Self {
        self.manifest_directory = Some(manifest_directory);
        self
    }

    pub fn catalog_schema_directory(mut self, catalog_schema_directory: String) -> Self {
        self.catalog_schema_directory = Some(catalog_schema_directory);
        self
    }

    pub fn catalogs_directory(mut self, catalogs_directory: String) -> Self {
        self.catalogs_directory = Some(catalogs_directory);
        self
    }

    pub fn build(self) -> RepositoryManager {
        RepositoryManager {
            register: self.register.expect("register undefined"),
            lease_id: self.lease_id.expect("lease id undefined"),
            app_directory: self.app_directory.expect("app directory undefined"),
            manifest_directory: self
                .manifest_directory
                .expect("manifest directory undefined"),
            catalog_schema_directory: self
                .catalog_schema_directory
                .expect("catalog schema directory undefined"),
            catalogs_directory: self
                .catalogs_directory
                .expect("catalogs directory undefined"),
        }
    }
}

pub struct RepositoryManager {
    register: Register,
    lease_id: i64,
    // app binary directory
    app_directory: String,
    // manifest file repository
    manifest_directory: String,
    // TODO: remote repository as backup
    // catalog schema repository
    catalog_schema_directory: String,
    // catalogs repository
    catalogs_directory: String,
}

impl RepositoryManager {
    pub fn builder() -> RepositoryManagerBuilder {
        RepositoryManagerBuilder::default()
    }

    pub async fn init(&self, reset: bool) -> pipebuilder_common::Result<()> {
        let app_directory = &self.app_directory;
        let manifest_directory = &self.manifest_directory;
        let catalog_schema_directory = &self.catalog_schema_directory;
        let catalogs_directory = &self.catalogs_directory;
        if reset {
            info!(path = app_directory.as_str(), "reset app directory");
            reset_directory(app_directory).await?;
            info!(
                path = manifest_directory.as_str(),
                "reset manifest directory"
            );
            reset_directory(manifest_directory).await?;
            info!(
                path = catalog_schema_directory.as_str(),
                "reset catalog schema directory"
            );
            reset_directory(catalog_schema_directory).await?;
            info!(
                path = catalogs_directory.as_str(),
                "reset catalogs directory"
            );
            reset_directory(catalogs_directory).await?;
        }
        Ok(())
    }

    pub async fn get_manifest(
        &self,
        resource: BlobDescriptor<'_>,
    ) -> pipebuilder_common::Result<Vec<u8>> {
        let repository = self.manifest_directory.as_str();
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        Self::read_resource::<ManifestMetadata>(
            repository,
            resource,
            TARGET_MANIFEST,
            &mut register,
            lease_id,
        )
        .await
    }

    pub async fn get_app(
        &self,
        resource: BlobDescriptor<'_>,
    ) -> pipebuilder_common::Result<Vec<u8>> {
        let repository = self.app_directory.as_str();
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        Self::read_resource::<AppMetadata>(
            repository,
            resource,
            TARGET_APP,
            &mut register,
            lease_id,
        )
        .await
    }

    pub async fn get_catalog_schema(
        &self,
        resource: BlobDescriptor<'_>,
    ) -> pipebuilder_common::Result<Vec<u8>> {
        let repository = self.catalog_schema_directory.as_str();
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        Self::read_resource::<CatalogSchemaMetadata>(
            repository,
            resource,
            TARGET_CATALOG_SCHEMA,
            &mut register,
            lease_id,
        )
        .await
    }

    pub async fn get_catalogs(
        &self,
        resource: BlobDescriptor<'_>,
    ) -> pipebuilder_common::Result<Vec<u8>> {
        let repository = self.catalogs_directory.as_str();
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        Self::read_resource::<CatalogsMetadata>(
            repository,
            resource,
            TARGET_CATALOGS,
            &mut register,
            lease_id,
        )
        .await
    }

    pub async fn put_manifest(
        &self,
        resource: SnapshotDescriptor<'_>,
        buffer: &[u8],
    ) -> pipebuilder_common::Result<u64> {
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let latest_version =
            Self::update_resource_snapshot::<ManifestSnapshot>(resource, &mut register, lease_id)
                .await?;
        let repository = self.manifest_directory.as_str();
        let (namespace, id) = resource.into_tuple();
        let resource = BlobDescriptor(namespace, id, latest_version);
        Self::write_resource::<ManifestMetadata>(
            repository,
            resource,
            TARGET_MANIFEST,
            buffer,
            &mut register,
            lease_id,
        )
        .await?;
        Ok(latest_version)
    }

    pub async fn post_app(
        &self,
        resource: BlobDescriptor<'_>,
        buffer: &[u8],
    ) -> pipebuilder_common::Result<()> {
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let repository = self.app_directory.as_str();
        Self::write_resource::<AppMetadata>(
            repository,
            resource,
            TARGET_APP,
            buffer,
            &mut register,
            lease_id,
        )
        .await
    }

    pub async fn put_catalog_schema(
        &self,
        resource: SnapshotDescriptor<'_>,
        buffer: &[u8],
    ) -> pipebuilder_common::Result<u64> {
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let latest_version = Self::update_resource_snapshot::<CatalogSchemaSnapshot>(
            resource,
            &mut register,
            lease_id,
        )
        .await?;
        let repository = self.catalog_schema_directory.as_str();
        let (namespace, id) = resource.into_tuple();
        let resource = BlobDescriptor(namespace, id, latest_version);
        Self::write_resource::<CatalogSchemaMetadata>(
            repository,
            resource,
            TARGET_CATALOG_SCHEMA,
            buffer,
            &mut register,
            lease_id,
        )
        .await?;
        Ok(latest_version)
    }

    pub async fn put_catalogs(
        &self,
        resource: SnapshotDescriptor<'_>,
        buffer: &[u8],
    ) -> pipebuilder_common::Result<u64> {
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let latest_version =
            Self::update_resource_snapshot::<CatalogsSnapshot>(resource, &mut register, lease_id)
                .await?;
        let repository = self.catalogs_directory.as_str();
        let (namespace, id) = resource.into_tuple();
        let resource = BlobDescriptor(namespace, id, latest_version);
        Self::write_resource::<CatalogsMetadata>(
            repository,
            resource,
            TARGET_CATALOGS,
            buffer,
            &mut register,
            lease_id,
        )
        .await?;
        Ok(latest_version)
    }

    pub async fn delete_manifest(
        &self,
        resource: BlobDescriptor<'_>,
    ) -> pipebuilder_common::Result<()> {
        let mut register = self.register.clone();
        let repository = self.manifest_directory.as_str();
        Self::delete_resource::<ManifestMetadata>(repository, resource, &mut register).await
    }

    pub async fn delete_app(&self, resource: BlobDescriptor<'_>) -> pipebuilder_common::Result<()> {
        let mut register = self.register.clone();
        let repository = self.app_directory.as_str();
        Self::delete_resource::<AppMetadata>(repository, resource, &mut register).await
    }

    pub async fn delete_catalog_schema(
        &self,
        resource: BlobDescriptor<'_>,
    ) -> pipebuilder_common::Result<()> {
        let mut register = self.register.clone();
        let repository = self.catalog_schema_directory.as_str();
        Self::delete_resource::<CatalogSchemaMetadata>(repository, resource, &mut register).await
    }

    pub async fn delete_catalogs(
        &self,
        resource: BlobDescriptor<'_>,
    ) -> pipebuilder_common::Result<()> {
        let mut register = self.register.clone();
        let repository = self.catalogs_directory.as_str();
        Self::delete_resource::<CatalogsMetadata>(repository, resource, &mut register).await
    }

    async fn read_resource<R>(
        repository: &str,
        resource: BlobDescriptor<'_>,
        target_name: &str,
        register: &mut Register,
        lease_id: i64,
    ) -> pipebuilder_common::Result<Vec<u8>>
    where
        R: Resource + BlobResource + Serialize + DeserializeOwned,
    {
        let (namespace, id, version) = resource.into_tuple();
        let buffer = match Self::read_target_from_repo(repository, resource, target_name).await {
            Ok(buffer) => buffer,
            Err(err) => {
                return Err(repository_error(
                    format!("read {}", R::ty()),
                    format!(
                        "read {} failed for (namespace = {}, id = {}, version = {}), error: {:#?}",
                        R::ty(),
                        namespace,
                        id,
                        version,
                        err
                    ),
                ))
            }
        };
        let size = buffer.len();
        match register
            .update_blob_resource::<R>(
                namespace,
                id,
                version,
                size,
                lease_id,
            )
            .await {
                Ok(_) => Ok(buffer),
                Err(err) => Err(repository_error(format!("update {} metadata", R::ty()), format!("update {} metadata failed for (namespace = {}, id = {}, version = {}), error: {:#?}", R::ty(), namespace, id, version, err))),
            }
    }

    // TODO: too many arguments
    async fn write_resource<R>(
        repository: &str,
        resource: BlobDescriptor<'_>,
        target_name: &str,
        buffer: &[u8],
        register: &mut Register,
        lease_id: i64,
    ) -> pipebuilder_common::Result<()>
    where
        R: Resource + BlobResource + Serialize + DeserializeOwned,
    {
        let (namespace, id, version) = resource.into_tuple();
        match Self::write_target_into_repo(repository, resource, buffer, target_name).await {
            Ok(_) => (),
            Err(err) => {
                return Err(repository_error(
                    format!("write {}", R::ty()),
                    format!(
                        "write {} failed for (namespace = {}, id = {}, version = {}), error: {:#?}",
                        R::ty(),
                        namespace,
                        id,
                        version,
                        err
                    ),
                ))
            }
        };
        let size = buffer.len();
        match register
            .update_blob_resource::<R>(
                namespace,
                id,
                version,
                size,
                lease_id,
            )
            .await {
                Ok(_) => Ok(()),
                Err(err) => Err(repository_error(format!("update {} metadata", R::ty()), format!("update {} metadata failed for (namespace = {}, id = {}, version = {}), error: {:#?}", R::ty(), namespace, id, version, err))),
            }
    }

    async fn delete_resource<R>(
        repository: &str,
        resource: BlobDescriptor<'_>,
        register: &mut Register,
    ) -> pipebuilder_common::Result<()>
    where
        R: Resource,
    {
        let (namespace, id, version) = resource.into_tuple();
        match Self::delete_target_from_repo(repository, resource) {
            Ok(_) => (),
            Err(err) => {
                return Err(repository_error(
                    format!("delete {}", R::ty()),
                    format!(
                    "delete {} failed for (namespace = {}, id = {}, version = {}), error: {:#?}",
                    R::ty(),
                    namespace,
                    id,
                    version,
                    err
                ),
                ))
            }
        };
        let (namespace, id, version) = resource.into_tuple();
        match register
            .delete_resource::<R>(Some(namespace), id, Some(version))
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(repository_error(
                format!("delete {} metadata", R::ty()),
                format!(
                    "delete {} failed for (namespace = {}, id = {}, version = {}), error: {:#?}",
                    R::ty(),
                    namespace,
                    id,
                    version,
                    err
                ),
            )),
        }
    }

    async fn update_resource_snapshot<S>(
        resource: SnapshotDescriptor<'_>,
        register: &mut Register,
        lease_id: i64,
    ) -> pipebuilder_common::Result<u64>
    where
        S: Resource + Snapshot + Serialize + DeserializeOwned,
    {
        let (namespace, id) = resource.into_tuple();
        match register
            .update_snapshot_resource::<S>(namespace, id, lease_id)
            .await
        {
            Ok((_, snapshot)) => Ok(snapshot.get_version()),
            Err(err) => Err(repository_error(
                format!("update {} snapshot", S::ty()),
                format!(
                    "update {} snapshot failed for (namespace = {}, id = {}), error: {:#?}",
                    S::ty(),
                    namespace,
                    id,
                    err
                ),
            )),
        }
    }

    async fn read_target_from_repo(
        repository: &str,
        resource: BlobDescriptor<'_>,
        target_name: &str,
    ) -> pipebuilder_common::Result<Vec<u8>> {
        let (namespace, id, version) = resource.into_tuple();
        let path = PathBuilder::default()
            .push(repository)
            .push(namespace)
            .push(id)
            .push(version.to_string())
            .push(target_name)
            .build();
        let buffer = read_file(path).await?;
        Ok(buffer)
    }

    async fn write_target_into_repo(
        repository: &str,
        resource: BlobDescriptor<'_>,
        buffer: &[u8],
        target_name: &str,
    ) -> pipebuilder_common::Result<()> {
        let (namespace, id, version) = resource.into_tuple();
        let directory = PathBuilder::default()
            .push(repository)
            .push(namespace)
            .push(id)
            .push(version.to_string())
            .build();
        let path = PathBuilder::clone_from(&directory)
            .push(target_name)
            .build();
        create_directory(directory.as_path()).await?;
        write_file(path.as_path(), buffer).await?;
        // TODO S3 backup
        Ok(())
    }

    fn delete_target_from_repo(
        repository: &str,
        resource: BlobDescriptor<'_>,
    ) -> pipebuilder_common::Result<()> {
        let (namespace, id, version) = resource.into_tuple();
        let directory = PathBuilder::default()
            .push(repository)
            .push(namespace)
            .push(id)
            .push(version.to_string())
            .build();
        remove_dir_all(directory.as_path())?;
        Ok(())
    }
}

pub struct RepositoryService {
    manager: RepositoryManager,
}

impl RepositoryService {
    pub fn new(manager: RepositoryManager) -> Self {
        RepositoryService { manager }
    }
}

#[tonic::async_trait]
impl Repository for RepositoryService {
    async fn get_manifest(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::repository::GetManifestRequest>,
    ) -> Result<
        tonic::Response<pipebuilder_common::grpc::repository::GetManifestResponse>,
        tonic::Status,
    > {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            manifest_version = version,
            "get manifest"
        );
        let resource = BlobDescriptor(namespace.as_str(), id.as_str(), version);
        match self.manager.get_manifest(resource).await {
            Ok(buffer) => Ok(Response::new(GetManifestResponse { buffer })),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    manifest_version = version,
                    "get manifest fail, error '{:#?}'",
                    err
                );
                Err(rpc_internal_error(err))
            }
        }
    }

    async fn put_manifest(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::repository::PutManifestRequest>,
    ) -> Result<
        tonic::Response<pipebuilder_common::grpc::repository::PutManifestResponse>,
        tonic::Status,
    > {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            "put manifest"
        );
        let buffer = request.buffer.as_slice();
        let resource = SnapshotDescriptor(namespace.as_str(), id.as_str());
        match self.manager.put_manifest(resource, buffer).await {
            Ok(version) => Ok(Response::new(PutManifestResponse { version })),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    "put manifest fail, error '{:#?}'",
                    err
                );
                Err(rpc_internal_error(err))
            }
        }
    }

    async fn delete_manifest(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::repository::DeleteManifestRequest>,
    ) -> Result<
        tonic::Response<pipebuilder_common::grpc::repository::DeleteManifestResponse>,
        tonic::Status,
    > {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            manifest_version = version,
            "delete manifest"
        );
        let resource = BlobDescriptor(namespace.as_str(), id.as_str(), version);
        match self.manager.delete_manifest(resource).await {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    manifest_version = version,
                    "delete manifest fail, error: '{:#?}'",
                    err
                )
                // return error ?
            }
        };
        Ok(Response::new(DeleteManifestResponse {}))
    }

    async fn get_app(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::repository::GetAppRequest>,
    ) -> Result<tonic::Response<pipebuilder_common::grpc::repository::GetAppResponse>, tonic::Status>
    {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            build_version = version,
            "get app"
        );
        let resource = BlobDescriptor(namespace.as_str(), id.as_str(), version);
        match self.manager.get_app(resource).await {
            Ok(buffer) => Ok(Response::new(GetAppResponse { buffer })),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    build_version = version,
                    "get app fail, error '{}'",
                    err
                );
                Err(rpc_internal_error(err))
            }
        }
    }

    async fn post_app(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::repository::PostAppRequest>,
    ) -> Result<tonic::Response<pipebuilder_common::grpc::repository::PostAppResponse>, tonic::Status>
    {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            build_version = version,
            "post app"
        );
        let buffer = request.buffer.as_slice();
        let resource = BlobDescriptor(namespace.as_str(), id.as_str(), version);
        match self.manager.post_app(resource, buffer).await {
            Ok(_) => Ok(Response::new(PostAppResponse {})),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    build_version = version,
                    "post app fail, error '{:#?}'",
                    err
                );
                Err(rpc_internal_error(err))
            }
        }
    }

    async fn delete_app(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::repository::DeleteAppRequest>,
    ) -> Result<
        tonic::Response<pipebuilder_common::grpc::repository::DeleteAppResponse>,
        tonic::Status,
    > {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            build_version = version,
            "delete app"
        );
        let resource = BlobDescriptor(namespace.as_str(), id.as_str(), version);
        match self.manager.delete_app(resource).await {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    build_version = version,
                    "delete app fail, error: '{:#?}'",
                    err
                )
                // return error ?
            }
        };
        Ok(Response::new(DeleteAppResponse {}))
    }

    async fn get_catalog_schema(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::repository::GetCatalogSchemaRequest>,
    ) -> Result<
        tonic::Response<pipebuilder_common::grpc::repository::GetCatalogSchemaResponse>,
        tonic::Status,
    > {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            catalog_schema_version = version,
            "get catalog schema"
        );
        let resource = BlobDescriptor(namespace.as_str(), id.as_str(), version);
        match self.manager.get_catalog_schema(resource).await {
            Ok(buffer) => Ok(Response::new(GetCatalogSchemaResponse { buffer })),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalog_schema_version = version,
                    "get catalog schema fail, error '{:#?}'",
                    err
                );
                Err(rpc_internal_error(err))
            }
        }
    }

    async fn put_catalog_schema(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::repository::PutCatalogSchemaRequest>,
    ) -> Result<
        tonic::Response<pipebuilder_common::grpc::repository::PutCatalogSchemaResponse>,
        tonic::Status,
    > {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            "get catalog schema"
        );
        let buffer = request.buffer.as_slice();
        let resource = SnapshotDescriptor(namespace.as_str(), id.as_str());
        match self.manager.put_catalog_schema(resource, buffer).await {
            Ok(version) => Ok(Response::new(PutCatalogSchemaResponse { version })),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    "put catalog schema fail, error '{:#?}'",
                    err
                );
                Err(rpc_internal_error(err))
            }
        }
    }

    async fn delete_catalog_schema(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::repository::DeleteCatalogSchemaRequest>,
    ) -> Result<
        tonic::Response<pipebuilder_common::grpc::repository::DeleteCatalogSchemaResponse>,
        tonic::Status,
    > {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            catalog_schema_version = version,
            "delete catalog schema"
        );
        let resource = BlobDescriptor(namespace.as_str(), id.as_str(), version);
        match self.manager.delete_catalog_schema(resource).await {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalog_schema_version = version,
                    "delete catalog schema fail, error: '{:#?}'",
                    err
                )
                // return error ?
            }
        };
        Ok(Response::new(DeleteCatalogSchemaResponse {}))
    }

    async fn get_catalogs(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::repository::GetCatalogsRequest>,
    ) -> Result<
        tonic::Response<pipebuilder_common::grpc::repository::GetCatalogsResponse>,
        tonic::Status,
    > {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            catalogs_version = version,
            "get catalogs"
        );
        let resource = BlobDescriptor(namespace.as_str(), id.as_str(), version);
        match self.manager.get_catalogs(resource).await {
            Ok(buffer) => Ok(Response::new(GetCatalogsResponse { buffer })),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalogs_version = version,
                    "get catalogs fail, error '{:#?}'",
                    err
                );
                Err(rpc_internal_error(err))
            }
        }
    }

    async fn put_catalogs(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::repository::PutCatalogsRequest>,
    ) -> Result<
        tonic::Response<pipebuilder_common::grpc::repository::PutCatalogsResponse>,
        tonic::Status,
    > {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            "get catalogs"
        );
        let buffer = request.buffer.as_slice();
        let resource = SnapshotDescriptor(namespace.as_str(), id.as_str());
        match self.manager.put_catalogs(resource, buffer).await {
            Ok(version) => Ok(Response::new(PutCatalogsResponse { version })),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    "put catalogs fail, error '{:#?}'",
                    err
                );
                Err(rpc_internal_error(err))
            }
        }
    }

    async fn delete_catalogs(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::repository::DeleteCatalogsRequest>,
    ) -> Result<
        tonic::Response<pipebuilder_common::grpc::repository::DeleteCatalogsResponse>,
        tonic::Status,
    > {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            catalogs_version = version,
            "delete catalogs"
        );
        let resource = BlobDescriptor(namespace.as_str(), id.as_str(), version);
        match self.manager.delete_catalogs(resource).await {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalogs_version = version,
                    "delete catalogs fail, error: '{:#?}'",
                    err
                )
                // return error ?
            }
        };
        Ok(Response::new(DeleteCatalogsResponse {}))
    }
}
