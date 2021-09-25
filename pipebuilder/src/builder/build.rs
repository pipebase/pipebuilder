use pipebuilder_common::{
    grpc::build::{builder_server::Builder, BuildRequest, BuildResponse},
    grpc::manifest::manifest_client::ManifestClient,
    Build, BuildStatus, Register, VersionBuild,
};
use tonic::{transport::Channel, Response};
use tracing::error;

pub struct BuilderService {
    lease_id: i64,
    register: Register,
    manifest_client: ManifestClient<Channel>,
    workspace: String,
    target_directory: String,
}

impl BuilderService {
    pub fn new(
        lease_id: i64,
        register: Register,
        manifest_client: ManifestClient<Channel>,
        workspace: String,
        target_directory: String,
    ) -> Self {
        BuilderService {
            lease_id,
            register,
            manifest_client,
            workspace,
            target_directory,
        }
    }
}

#[tonic::async_trait]
impl Builder for BuilderService {
    async fn build(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::build::BuildRequest>,
    ) -> Result<tonic::Response<pipebuilder_common::grpc::build::BuildResponse>, tonic::Status>
    {
        let request = request.into_inner();
        let manifest_id = request.manifest_id;
        // lock build snapshot with manifest id
        // update latest build version
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let snapshot = match register
            .incr_build_snapshot(manifest_id.as_str(), lease_id)
            .await
        {
            Ok((_, snapshot)) => snapshot,
            Err(err) => {
                error!("trigger build failed, error: {:#?}", err);
                return Err(tonic::Status::internal(format!("{:#?}", err)));
            }
        };
        let build_version = snapshot.latest_version;
        // fetch latest manifest
        let manifest_client = self.manifest_client.clone();
        /*
        let (app, manifest_version) =
            match read_manifest(manifest_client, String::from(manifest_id)).await {
                Ok((app, version)) => (app, version),
                Err(err) => {
                    error!("read manifest {} failed, error: {:#?}", manifest_id, err);
                    return Err(tonic::Status::invalid_argument(format!("{:#?}", err)));
                }
            };
            */
        let workspace = self.workspace.to_owned();
        let target_directory = self.target_directory.to_owned();
        let target_platform = request.target_platform;
        let build = Build::new(
            manifest_id,
            manifest_client,
            build_version,
            workspace,
            target_directory,
            target_platform,
        );
        // trigger build
        // register.put_version_build_state(&id.to_string(), version, BuildStatus::Create, lease_id)
        // response
        Ok(Response::new(BuildResponse {
            version: build_version,
        }))
    }
}

async fn fail(
    mut register: Register,
    lease_id: i64,
    build: Build,
    message: String,
) -> pipebuilder_common::Result<()> {
    let id = build.get_id();
    let version = build.get_build_version();
    register
        .put_version_build_state(lease_id, &id, version, BuildStatus::Fail, message.into())
        .await?;
    Ok(())
}
