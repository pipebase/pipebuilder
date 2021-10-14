use pipebuilder_common::{
    create_directory,
    grpc::repository::{
        repository_server::Repository, GetAppResponse, GetManifestResponse, PostAppResponse,
        PutManifestResponse,
    },
    read_file, rpc_internal_error, rpc_not_found, write_file, Register,
};
use tonic::Response;
use tracing::error;

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
        let repository = self.manifest_repository.as_str();
        match read_target_from_repo(
            repository,
            namespace.as_str(),
            id.as_str(),
            version,
            TARGET_MANIFEST,
        ) {
            Ok(buffer) => Ok(Response::new(GetManifestResponse { buffer })),
            Err(err) => {
                error!(
                    "read manifest {}/{}/{} fail, error '{}'",
                    namespace, id, version, err
                );
                Err(rpc_not_found("manifest not found"))
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
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let version = match register
            .incr_manifest_snapshot(lease_id, namespace.as_str(), id.as_str())
            .await
        {
            Ok((_, snapshot)) => snapshot.latest_version,
            Err(err) => {
                error!("increase manifest snapshot version fail, error '{}'", err);
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
        ) {
            Ok(_) => Ok(Response::new(PutManifestResponse { id, version })),
            Err(err) => {
                error!("write manifest fail, error '{}'", err);
                return Err(rpc_internal_error(err));
            }
        }
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
        let repository = self.app_repository.as_str();
        match read_target_from_repo(
            repository,
            namespace.as_str(),
            id.as_str(),
            version,
            TARGET_APP,
        ) {
            Ok(buffer) => Ok(Response::new(GetAppResponse { buffer })),
            Err(err) => {
                error!(
                    "read app {}/{}/{} fail, error '{}'",
                    namespace, id, version, err
                );
                Err(rpc_not_found("app not found"))
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
        let buffer = request.buffer.as_slice();
        let repository = self.app_repository.as_str();
        match write_target_into_repo(
            repository,
            namespace.as_str(),
            id.as_str(),
            version,
            buffer,
            TARGET_APP,
        ) {
            Ok(_) => Ok(Response::new(PostAppResponse {})),
            Err(err) => {
                error!("post app fail, error '{}'", err);
                return Err(rpc_internal_error(err));
            }
        }
    }
}

fn read_target_from_repo(
    repository: &str,
    namespace: &str,
    id: &str,
    version: u64,
    target_name: &str,
) -> pipebuilder_common::Result<Vec<u8>> {
    let path = get_target_path(repository, namespace, id, version, target_name);
    let buffer = read_file(path)?;
    Ok(buffer)
}

fn write_target_into_repo(
    repository: &str,
    namespace: &str,
    id: &str,
    version: u64,
    buffer: &[u8],
    target_name: &str,
) -> pipebuilder_common::Result<()> {
    let path = get_target_path(repository, namespace, id, version, target_name);
    let directory = get_target_directory(repository, namespace, id, version);
    create_directory(directory)?;
    write_file(path, buffer)?;
    // TODO S3 backup
    Ok(())
}

fn get_target_directory(repository: &str, namespace: &str, id: &str, version: u64) -> String {
    format!("{}/{}/{}/{}", repository, namespace, id, version)
}

fn get_target_path(
    repository: &str,
    namespace: &str,
    id: &str,
    version: u64,
    target_name: &str,
) -> String {
    format!(
        "{}/{}/{}/{}/{}",
        repository, namespace, id, version, target_name
    )
}
