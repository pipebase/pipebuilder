use pipebuilder_common::{
    self, create_directory,
    grpc::repository::{
        repository_server::Repository, DeleteAppResponse, DeleteCatalogSchemaResponse,
        DeleteCatalogsResponse, DeleteManifestResponse, GetAppResponse, GetCatalogSchemaResponse,
        GetCatalogsResponse, GetManifestResponse, PostAppResponse, PutCatalogSchemaResponse,
        PutCatalogsResponse, PutManifestResponse,
    },
    read_file, reset_directory, rpc_internal_error, rpc_not_found, sub_path, write_file,
    AppMetadata, CatalogSchemaMetadata, CatalogSchemaSnapshot, CatalogsMetadata, CatalogsSnapshot,
    ManifestMetadata, ManifestSnapshot, Register,
};
use std::fs::remove_dir_all;
use tonic::Response;
use tracing::{error, info};

pub const TARGET_MANIFEST: &str = "pipe.yml";
pub const TARGET_APP: &str = "app";
pub const TARGET_CATALOG_SCHEMA: &str = "schema.yml";
pub const TARGET_CATALOGS: &str = "catalogs.yml";

#[derive(Default)]
pub struct RepositoryServiceBuilder {
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

impl RepositoryServiceBuilder {
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

    pub fn build(self) -> RepositoryService {
        RepositoryService {
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

pub struct RepositoryService {
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

impl RepositoryService {
    pub fn builder() -> RepositoryServiceBuilder {
        RepositoryServiceBuilder::default()
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
        let repository = self.manifest_directory.as_str();
        let buffer = match read_target_from_repo(
            repository,
            namespace.as_str(),
            id.as_str(),
            version,
            TARGET_MANIFEST,
        )
        .await
        {
            Ok(buffer) => buffer,
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    manifest_version = version,
                    "read manifest fail, error '{:#?}'",
                    err
                );
                return Err(rpc_not_found("manifest not found"));
            }
        };
        // update manifest metadata
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let size = buffer.len();
        match register
            .update_blob_resource::<ManifestMetadata>(
                namespace.as_str(),
                id.as_str(),
                version,
                size,
                lease_id,
            )
            .await
        {
            Ok(_) => Ok(Response::new(GetManifestResponse { buffer })),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    manifest_version = version,
                    "update manifest metadata fail, error '{:#?}'",
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
            "get manifest"
        );
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let version = match register
            .update_snapshot_resource::<ManifestSnapshot>(namespace.as_str(), id.as_str(), lease_id)
            .await
        {
            Ok((_, snapshot)) => snapshot.latest_version,
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    "increase manifest snapshot version fail, error '{:#?}'",
                    err
                );
                return Err(rpc_internal_error(err));
            }
        };
        let repository = self.manifest_directory.as_str();
        let buffer = request.buffer.as_slice();
        match write_target_into_repo(
            repository,
            namespace.as_str(),
            id.as_str(),
            version,
            buffer,
            TARGET_MANIFEST,
        )
        .await
        {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    manifest_version = version,
                    "write manifest fail, error '{:#?}'",
                    err
                );
                return Err(rpc_internal_error(err));
            }
        };
        // update manifest metadata
        let size = buffer.len();
        match register
            .update_blob_resource::<ManifestMetadata>(
                namespace.as_str(),
                id.as_str(),
                version,
                size,
                lease_id,
            )
            .await
        {
            Ok(_) => Ok(Response::new(PutManifestResponse { version })),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    manifest_version = version,
                    "update manifest metadata fail, error '{:#?}'",
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
        let repository = self.manifest_directory.as_str();
        match delete_target_from_repo(repository, namespace.as_str(), id.as_str(), version) {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    manifest_version = version,
                    "delete manifest fail, error: '{:#?}'",
                    err
                );
                // return error ?
            }
        };
        let mut register = self.register.clone();
        match register
            .delete_resource::<ManifestMetadata>(
                Some(namespace.as_str()),
                id.as_str(),
                Some(version),
            )
            .await
        {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    manifest_version = version,
                    "delete manifest metadata fail, error: '{:#?}'",
                    err
                )
                // return error ?
            }
        }
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
        let repository = self.app_directory.as_str();
        let buffer = match read_target_from_repo(
            repository,
            namespace.as_str(),
            id.as_str(),
            version,
            TARGET_APP,
        )
        .await
        {
            Ok(buffer) => buffer,
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    build_version = version,
                    "read app fail, error '{:#?}'",
                    err
                );
                return Err(rpc_not_found("app not found"));
            }
        };
        // update app metadata
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let size = buffer.len();
        match register
            .update_blob_resource::<AppMetadata>(
                namespace.as_str(),
                id.as_str(),
                version,
                size,
                lease_id,
            )
            .await
        {
            Ok(_) => Ok(Response::new(GetAppResponse { buffer })),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    build_version = version,
                    "update app metadata fail, error '{}'",
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
        let repository = self.app_directory.as_str();
        match write_target_into_repo(
            repository,
            namespace.as_str(),
            id.as_str(),
            version,
            buffer,
            TARGET_APP,
        )
        .await
        {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    build_version = version,
                    "post app fail, error '{:#?}'",
                    err
                );
                return Err(rpc_internal_error(err));
            }
        };
        // update app metadata
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let size = buffer.len();
        match register
            .update_blob_resource::<AppMetadata>(
                namespace.as_str(),
                id.as_str(),
                version,
                size,
                lease_id,
            )
            .await
        {
            Ok(_) => Ok(Response::new(PostAppResponse {})),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    build_version = version,
                    "update app metadata fail, error '{:#?}'",
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
        let repository = self.app_directory.as_str();
        match delete_target_from_repo(repository, namespace.as_str(), id.as_str(), version) {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    build_version = version,
                    "delete app fail, error: '{:#?}'",
                    err
                );
                // return error ?
            }
        };
        let mut register = self.register.clone();
        match register
            .delete_resource::<AppMetadata>(Some(namespace.as_str()), id.as_str(), Some(version))
            .await
        {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    build_version = version,
                    "delete app metadata fail, error: '{:#?}'",
                    err
                )
                // return error ?
            }
        }
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
        let repository = self.catalog_schema_directory.as_str();
        let buffer = match read_target_from_repo(
            repository,
            namespace.as_str(),
            id.as_str(),
            version,
            TARGET_CATALOG_SCHEMA,
        )
        .await
        {
            Ok(buffer) => buffer,
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalog_schema_version = version,
                    "read catalog schema fail, error '{:#?}'",
                    err
                );
                return Err(rpc_not_found("catalog schema not found"));
            }
        };
        // update catalog schema metadata
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let size = buffer.len();
        match register
            .update_blob_resource::<CatalogSchemaMetadata>(
                namespace.as_str(),
                id.as_str(),
                version,
                size,
                lease_id,
            )
            .await
        {
            Ok(_) => Ok(Response::new(GetCatalogSchemaResponse { buffer })),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalog_schema_version = version,
                    "update catalog schema metadata fail, error '{:#?}'",
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
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let version = match register
            .update_snapshot_resource::<CatalogSchemaSnapshot>(
                namespace.as_str(),
                id.as_str(),
                lease_id,
            )
            .await
        {
            Ok((_, snapshot)) => snapshot.latest_version,
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    "increase catalog schema snapshot version fail, error '{:#?}'",
                    err
                );
                return Err(rpc_internal_error(err));
            }
        };
        let repository = self.catalog_schema_directory.as_str();
        let buffer = request.buffer.as_slice();
        match write_target_into_repo(
            repository,
            namespace.as_str(),
            id.as_str(),
            version,
            buffer,
            TARGET_CATALOG_SCHEMA,
        )
        .await
        {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalog_schema_version = version,
                    "write catalog schema fail, error '{:#?}'",
                    err
                );
                return Err(rpc_internal_error(err));
            }
        };
        // update catalog schema metadata
        let size = buffer.len();
        match register
            .update_blob_resource::<CatalogSchemaMetadata>(
                namespace.as_str(),
                id.as_str(),
                version,
                size,
                lease_id,
            )
            .await
        {
            Ok(_) => Ok(Response::new(PutCatalogSchemaResponse { version })),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalog_schema_version = version,
                    "update catalog schema metadata fail, error '{:#?}'",
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
        let repository = self.catalog_schema_directory.as_str();
        match delete_target_from_repo(repository, namespace.as_str(), id.as_str(), version) {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalog_schema_version = version,
                    "delete catalog schema fail, error: '{:#?}'",
                    err
                );
                // return error ?
            }
        };
        let mut register = self.register.clone();
        match register
            .delete_resource::<CatalogSchemaMetadata>(
                Some(namespace.as_str()),
                id.as_str(),
                Some(version),
            )
            .await
        {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalog_schema_version = version,
                    "delete catalog schema metadata fail, error: '{:#?}'",
                    err
                )
                // return error ?
            }
        }
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
        let repository = self.catalogs_directory.as_str();
        let buffer = match read_target_from_repo(
            repository,
            namespace.as_str(),
            id.as_str(),
            version,
            TARGET_CATALOGS,
        )
        .await
        {
            Ok(buffer) => buffer,
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalogs_version = version,
                    "read catalogs fail, error '{:#?}'",
                    err
                );
                return Err(rpc_not_found("catalogs not found"));
            }
        };
        // update catalogs metadata
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let size = buffer.len();
        match register
            .update_blob_resource::<CatalogsMetadata>(
                namespace.as_str(),
                id.as_str(),
                version,
                size,
                lease_id,
            )
            .await
        {
            Ok(_) => Ok(Response::new(GetCatalogsResponse { buffer })),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalogs_version = version,
                    "update catalogs metadata fail, error '{:#?}'",
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
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let version = match register
            .update_snapshot_resource::<CatalogsSnapshot>(namespace.as_str(), id.as_str(), lease_id)
            .await
        {
            Ok((_, snapshot)) => snapshot.latest_version,
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    "increase catalogs snapshot version fail, error '{:#?}'",
                    err
                );
                return Err(rpc_internal_error(err));
            }
        };
        let repository = self.catalogs_directory.as_str();
        let buffer = request.buffer.as_slice();
        match write_target_into_repo(
            repository,
            namespace.as_str(),
            id.as_str(),
            version,
            buffer,
            TARGET_CATALOGS,
        )
        .await
        {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalogs_version = version,
                    "write catalogs fail, error '{:#?}'",
                    err
                );
                return Err(rpc_internal_error(err));
            }
        };
        // update catalogs metadata
        let size = buffer.len();
        match register
            .update_blob_resource::<CatalogsMetadata>(
                namespace.as_str(),
                id.as_str(),
                version,
                size,
                lease_id,
            )
            .await
        {
            Ok(_) => Ok(Response::new(PutCatalogsResponse { version })),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalogs_version = version,
                    "update catalogs metadata fail, error '{:#?}'",
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
            "delete catalog schema"
        );
        let repository = self.catalogs_directory.as_str();
        match delete_target_from_repo(repository, namespace.as_str(), id.as_str(), version) {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalogs_version = version,
                    "delete catalogs fail, error: '{:#?}'",
                    err
                );
                // return error ?
            }
        };
        let mut register = self.register.clone();
        match register
            .delete_resource::<CatalogsMetadata>(
                Some(namespace.as_str()),
                id.as_str(),
                Some(version),
            )
            .await
        {
            Ok(_) => (),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    catalogs_version = version,
                    "delete catalogs metadata fail, error: '{:#?}'",
                    err
                )
                // return error ?
            }
        }
        Ok(Response::new(DeleteCatalogsResponse {}))
    }
}

async fn read_target_from_repo(
    repository: &str,
    namespace: &str,
    id: &str,
    version: u64,
    target_name: &str,
) -> pipebuilder_common::Result<Vec<u8>> {
    let directory = get_target_directory(repository, namespace, id, version);
    let path = sub_path(directory.as_str(), target_name);
    let buffer = read_file(path).await?;
    Ok(buffer)
}

async fn write_target_into_repo(
    repository: &str,
    namespace: &str,
    id: &str,
    version: u64,
    buffer: &[u8],
    target_name: &str,
) -> pipebuilder_common::Result<()> {
    let directory = get_target_directory(repository, namespace, id, version);
    let path = sub_path(directory.as_str(), target_name);
    create_directory(directory).await?;
    write_file(path, buffer).await?;
    // TODO S3 backup
    Ok(())
}

fn delete_target_from_repo(
    repository: &str,
    namespace: &str,
    id: &str,
    version: u64,
) -> pipebuilder_common::Result<()> {
    let directory = get_target_directory(repository, namespace, id, version);
    remove_dir_all(directory)?;
    Ok(())
}

fn get_target_directory(repository: &str, namespace: &str, id: &str, version: u64) -> String {
    format!("{}/{}/{}/{}", repository, namespace, id, version)
}
