use pipebuilder_common::{
    create_directory,
    grpc::manifest::{manifest_server::Manifest, GetManifestResponse, PutManifestResponse},
    read_file, rpc_internal_error, rpc_not_found, write_file, Register,
};
use tonic::Response;
use tracing::error;

pub struct ManifestService {
    register: Register,
    lease_id: i64,
    // repository of local filesystem
    repository: String,
    // TODO: remote repository as backup
}

impl ManifestService {
    pub fn new(register: Register, lease_id: i64, repository: String) -> Self {
        ManifestService {
            register,
            lease_id,
            repository,
        }
    }
}

#[tonic::async_trait]
impl Manifest for ManifestService {
    async fn get_manifest(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::manifest::GetManifestRequest>,
    ) -> Result<
        tonic::Response<pipebuilder_common::grpc::manifest::GetManifestResponse>,
        tonic::Status,
    > {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        let version = request.version;
        let repository = self.repository.as_str();
        match read_manifest_from_repo(repository, namespace.as_str(), id.as_str(), version) {
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
        request: tonic::Request<pipebuilder_common::grpc::manifest::PutManifestRequest>,
    ) -> Result<
        tonic::Response<pipebuilder_common::grpc::manifest::PutManifestResponse>,
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
        let repository = self.repository.as_str();
        let buffer = request.buffer.as_slice();
        match write_manifest_into_repo(repository, namespace.as_str(), id.as_str(), version, buffer)
        {
            Ok(_) => Ok(Response::new(PutManifestResponse { id, version })),
            Err(err) => {
                error!("write manifest fail, error '{}'", err);
                return Err(rpc_internal_error(err));
            }
        }
    }
}

fn read_manifest_from_repo(
    repository: &str,
    namespace: &str,
    id: &str,
    version: u64,
) -> pipebuilder_common::Result<Vec<u8>> {
    let path = get_manifest_path(repository, namespace, id, version);
    let buffer = read_file(path)?;
    Ok(buffer)
}

fn write_manifest_into_repo(
    repository: &str,
    namespace: &str,
    id: &str,
    version: u64,
    buffer: &[u8],
) -> pipebuilder_common::Result<()> {
    let path = get_manifest_path(repository, namespace, id, version);
    let directory = get_manifest_directory(repository, namespace, id, version);
    create_directory(directory)?;
    write_file(path, buffer)?;
    // TODO S3 backup
    Ok(())
}

fn get_manifest_directory(repository: &str, namespace: &str, id: &str, version: u64) -> String {
    format!("{}/{}/{}/{}", repository, namespace, id, version)
}

fn get_manifest_path(repository: &str, namespace: &str, id: &str, version: u64) -> String {
    format!("{}/{}/{}/{}/pipe.yml", repository, namespace, id, version)
}
