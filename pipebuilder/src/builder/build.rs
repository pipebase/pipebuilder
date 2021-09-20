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
        let manifest_url = request.manifest_url.as_str();
        // lock build snapshot with manifest url
        // update latest version
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let snapshot = match register.incr_build_snapshot(manifest_url, lease_id).await {
            Ok((_, snapshot)) => snapshot,
            Err(err) => {
                error!("trigger build failed, error: {:#?}", err);
                return Err(tonic::Status::internal(format!("{:#?}", err)));
            }
        };
        let id = snapshot.id;
        let version = snapshot.latest_version;
        let manifest_url = String::from(manifest_url);
        let build = Build::new(manifest_url, id, version);
        // trigger version build
        // register.put_version_build_state(&id.to_string(), version, BuildStatus::Create, lease_id)
        // response
        Ok(Response::new(BuildResponse { version }))
    }
}

// build state machine
#[async_recursion]
async fn do_build(register: Register, lease_id: i64, build: Build, status: BuildStatus) {
    match status {
        BuildStatus::Create => do_build(register, lease_id, build, BuildStatus::Pull).await,
        BuildStatus::Pull => do_build(register, lease_id, build, BuildStatus::Validate).await,
        BuildStatus::Validate => do_build(register, lease_id, build, BuildStatus::Initialize).await,
        BuildStatus::Initialize => do_build(register, lease_id, build, BuildStatus::Generate).await,
        BuildStatus::Generate => do_build(register, lease_id, build, BuildStatus::Build).await,
        BuildStatus::Build => do_build(register, lease_id, build, BuildStatus::Store).await,
        BuildStatus::Store => do_build(register, lease_id, build, BuildStatus::Publish).await,
        BuildStatus::Publish => do_build(register, lease_id, build, BuildStatus::Done).await,
        BuildStatus::Done => {}
        _ => unreachable!(),
    };
}

async fn do_fail(
    mut register: Register,
    lease_id: i64,
    build: Build,
    message: String,
) -> pipebuilder_common::Result<()> {
    let id = build.get_string_id();
    let version = build.get_version();
    register
        .put_version_build_state(lease_id, &id, version, BuildStatus::Fail, message.into())
        .await?;
    Ok(())
}
