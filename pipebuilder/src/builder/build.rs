use async_recursion::async_recursion;
use pipebuilder_common::{
    grpc::build::{builder_server::Builder, BuildRequest, BuildResponse},
    Build, BuildStatus, Register, VersionBuild,
};
use tonic::Response;
use tracing::error;

pub struct BuilderService {
    lease_id: i64,
    register: Register,
}

#[tonic::async_trait]
impl Builder for BuilderService {
    async fn build(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::build::BuildRequest>,
    ) -> Result<tonic::Response<pipebuilder_common::grpc::build::BuildResponse>, tonic::Status>
    {
        let request = request.get_ref();
        let manifest_id = request.manifest_id.as_str();
        // lock build snapshot with manifest id
        // update latest version
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let snapshot = match register.incr_build_snapshot(manifest_id, lease_id).await {
            Ok((_, snapshot)) => snapshot,
            Err(err) => {
                error!("trigger build failed, error: {:#?}", err);
                return Err(tonic::Status::internal(format!("{:#?}", err)));
            }
        };
        let version = snapshot.latest_version;
        let manifest_id = String::from(manifest_id);
        let build = Build::new(manifest_id, version);
        // trigger version build
        // register.put_version_build_state(&id.to_string(), version, BuildStatus::Create, lease_id)
        // response
        Ok(Response::new(BuildResponse { version }))
    }
}

// build state machine
#[async_recursion]
async fn run(register: Register, lease_id: i64, build: Build, status: BuildStatus) {
    match status {
        BuildStatus::Create => run(register, lease_id, build, BuildStatus::Pull).await,
        BuildStatus::Pull => run(register, lease_id, build, BuildStatus::Validate).await,
        BuildStatus::Validate => run(register, lease_id, build, BuildStatus::Initialize).await,
        BuildStatus::Initialize => run(register, lease_id, build, BuildStatus::Generate).await,
        BuildStatus::Generate => run(register, lease_id, build, BuildStatus::Build).await,
        BuildStatus::Build => run(register, lease_id, build, BuildStatus::Store).await,
        BuildStatus::Store => run(register, lease_id, build, BuildStatus::Publish).await,
        BuildStatus::Publish => run(register, lease_id, build, BuildStatus::Done).await,
        BuildStatus::Done => {}
        _ => unreachable!(),
    };
}

async fn fail(
    mut register: Register,
    lease_id: i64,
    build: Build,
    message: String,
) -> pipebuilder_common::Result<()> {
    let id = build.get_id();
    let version = build.get_version();
    register
        .put_version_build_state(lease_id, &id, version, BuildStatus::Fail, message.into())
        .await?;
    Ok(())
}
