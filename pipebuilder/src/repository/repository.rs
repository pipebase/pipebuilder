use pipebuilder_common::{
    create_directory,
    grpc::repository::{
        repository_server::Repository, DeleteAppResponse, DeleteManifestResponse, GetAppResponse,
        GetManifestResponse, PostAppResponse, PutManifestResponse,
    },
    read_file, rpc_internal_error, rpc_not_found, sub_path, write_file, Register,
};
use std::fs::remove_dir_all;
use tonic::Response;
use tracing::{error, info};

pub const TARGET_MANIFEST: &str = "pipe.yml";
pub const TARGET_APP: &str = "app";

pub struct RepositoryService {
    register: Register,
    lease_id: i64,
    // app binary repository
    app_repository: String,
    // manifest file repository
    manifest_repository: String,
    // TODO: remote repository as backup
}

impl RepositoryService {
    pub fn new(
        register: Register,
        lease_id: i64,
        app_repository: String,
        manifest_repository: String,
    ) -> Self {
        RepositoryService {
            register,
            lease_id,
            app_repository,
            manifest_repository,
        }
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
        let repository = self.manifest_repository.as_str();
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
            .update_manifest_metadata(lease_id, namespace.as_str(), id.as_str(), version, size)
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
            .incr_manifest_snapshot(lease_id, namespace.as_str(), id.as_str())
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
        let repository = self.manifest_repository.as_str();
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
            .update_manifest_metadata(lease_id, namespace.as_str(), id.as_str(), version, size)
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
        let repository = self.manifest_repository.as_str();
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
            .delete_manifest_metadata(namespace.as_str(), id.as_str(), version)
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
        let repository = self.app_repository.as_str();
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
            .update_app_metadata(lease_id, namespace.as_str(), id.as_str(), version, size)
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
        let repository = self.app_repository.as_str();
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
            .update_app_metadata(lease_id, namespace.as_str(), id.as_str(), version, size)
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
        let repository = self.app_repository.as_str();
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
            .delete_app_metadata(namespace.as_str(), id.as_str(), version)
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
