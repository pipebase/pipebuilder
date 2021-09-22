use pipebuilder_common::{
    grpc::manifest::{manifest_server::Manifest, GetManifestResponse, PutManifestResponse},
    internal_error, not_found, read_file, write_file, Register,
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
        let id = request_ref.id.as_str();
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let snapshot = match register.get_manifest_snapshot(lease_id, id).await {
            Ok(snapshot) => snapshot,
            Err(err) => return Err(internal_error(err)),
        };
        let snapshot = match snapshot {
            Some(snapshot) => snapshot,
            None => return Err(not_found(&format!("manifest {} not found in register", id))),
        };
        // TODO: read manifest binaries from repository
        let repository = self.repository.as_str();
        let version = snapshot.latest_version;
        match read_manifest_from_repo(repository, id, version) {
            Ok(buffer) => Ok(Response::new(GetManifestResponse { version, buffer })),
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
        let id = match request_ref.id {
            Some(ref id) => id.to_owned(),
            None => {
                let uuid = Uuid::new_v4();
                uuid.to_string()
            }
        };
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let version = match register.incr_manifest_snapshot(lease_id, id.as_str()).await {
            Ok((_, snapshot)) => snapshot.latest_version,
            Err(err) => return Err(internal_error(err)),
        };
        let repository = self.repository.as_str();
        let buffer = request_ref.buffer.as_slice();
        match write_manifest_into_repo(repository, id.as_str(), version, buffer) {
            Ok(_) => Ok(Response::new(PutManifestResponse { id, version })),
            Err(err) => return Err(internal_error(err)),
        }
    }
}

fn read_manifest_from_repo(
    repository: &str,
    id: &str,
    version: u64,
) -> pipebuilder_common::Result<Vec<u8>> {
    let path = get_manifest_path(repository, id, version);
    let buffer = read_file(path)?;
    Ok(buffer)
}

fn write_manifest_into_repo(
    repository: &str,
    id: &str,
    version: u64,
    buffer: &[u8],
) -> pipebuilder_common::Result<()> {
    let path = get_manifest_path(repository, id, version);
    write_file(path, buffer)?;
    Ok(())
}

fn get_manifest_path(repository: &str, id: &str, version: u64) -> String {
    format!("{}/{}/{}", repository, id, version)
}
