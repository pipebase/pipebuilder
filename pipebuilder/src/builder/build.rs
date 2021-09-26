use chrono::Utc;
use pipebuilder_common::{
    grpc::build::{builder_server::Builder, BuildResponse},
    grpc::manifest::manifest_client::ManifestClient,
    Build, BuildStatus, LocalBuildContext, Register, VersionBuild,
};
use tonic::{transport::Channel, Response};
use tracing::error;

pub struct BuilderService {
    lease_id: i64,
    register: Register,
    manifest_client: ManifestClient<Channel>,
    context: LocalBuildContext,
}

impl BuilderService {
    pub fn new(
        lease_id: i64,
        register: Register,
        manifest_client: ManifestClient<Channel>,
        context: LocalBuildContext,
    ) -> Self {
        BuilderService {
            lease_id,
            register,
            manifest_client,
            context,
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
        // prepare build contexts
        let build_version = snapshot.latest_version;
        let manifest_client = self.manifest_client.clone();
        let build_context = self.context.to_owned();
        let target_platform = request.target_platform;
        let build = Build::new(
            manifest_id,
            manifest_client,
            build_version,
            build_context,
            target_platform,
        );
        // trigger build
        let lease_id = self.lease_id;
        let register = self.register.to_owned();
        run(lease_id, register, build);
        Ok(Response::new(BuildResponse {
            version: build_version,
        }))
    }
}

fn run(lease_id: i64, mut register: Register, mut build: Build) {
    let _ = tokio::spawn(async move {
        let mut status = BuildStatus::Pull;
        loop {
            // update build status in register
            match update(&mut register, lease_id, &build, status.clone(), None).await {
                Ok(()) => (),
                Err(err) => {
                    error!(
                        "update build status failed for {}, error: {:#?}",
                        build.get_id(),
                        err
                    );
                    return;
                }
            };
            // run current build state
            let result = build.run(status.clone()).await;
            let next_status = match result {
                Ok(next_status) => next_status,
                Err(err) => {
                    let _ = update(
                        &mut register,
                        lease_id,
                        &build,
                        BuildStatus::Fail,
                        Some(format!("{:#?}", err)),
                    )
                    .await;
                    return;
                }
            };
            // continue next state or exit
            match next_status {
                Some(next_status) => status = next_status,
                None => return,
            }
        }
    });
}

// update version build status
async fn update(
    register: &mut Register,
    lease_id: i64,
    build: &Build,
    status: BuildStatus,
    message: Option<String>,
) -> pipebuilder_common::Result<()> {
    let id = build.get_id();
    let version = build.get_build_version();
    let (builder_id, builder_address) = build.get_builder_info();
    let now = Utc::now();
    let state = VersionBuild::new(
        status,
        now,
        builder_id.to_owned(),
        builder_address.to_owned(),
        message,
    );
    register
        .put_version_build_state(lease_id, &id, version, state)
        .await?;
    Ok(())
}
