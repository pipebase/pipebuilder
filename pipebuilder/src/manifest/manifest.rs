use pipebuilder_common::{
    grpc::manifest::{manifest_server::Manifest, GetManifestResponse, PutManifestResponse},
    internal_error, read_file, write_file, Register,
};
use tonic::Response;
use uuid::Uuid;

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
        let request_ref = request.get_ref();
        let repository = self.repository.as_str();
        let namespace = request_ref.namespace.as_str();
        let id = request_ref.id.as_str();
        let version = request_ref.version;
        match read_manifest_from_repo(repository, namespace, id, version) {
            Ok(buffer) => Ok(Response::new(GetManifestResponse { buffer })),
            Err(err) => Err(internal_error(err)),
        }
    }

    async fn put_manifest(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::manifest::PutManifestRequest>,
    ) -> Result<
        tonic::Response<pipebuilder_common::grpc::manifest::PutManifestResponse>,
        tonic::Status,
    > {
        let request_ref = request.get_ref();
        let namespace = request_ref.namespace.as_str();
        let id = match request_ref.id {
            Some(ref id) => id.to_owned(),
            None => {
                let uuid = Uuid::new_v4();
                uuid.to_string()
            }
        };
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let version = match register
            .incr_manifest_snapshot(lease_id, namespace, id.as_str())
            .await
        {
            Ok((_, snapshot)) => snapshot.latest_version,
            Err(err) => return Err(internal_error(err)),
        };
        let repository = self.repository.as_str();
        let buffer = request_ref.buffer.as_slice();
        // TODO: validate manifest before write
        match write_manifest_into_repo(repository, namespace, id.as_str(), version, buffer) {
            Ok(_) => Ok(Response::new(PutManifestResponse { id, version })),
            Err(err) => return Err(internal_error(err)),
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
    write_file(path, buffer)?;
    // TODO S3 backup
    Ok(())
}

fn get_manifest_path(repository: &str, namespace: &str, id: &str, version: u64) -> String {
    format!("{}/{}/{}/{}", repository, namespace, id, version)
}
