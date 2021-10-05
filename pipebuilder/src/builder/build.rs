use chrono::Utc;
use flurry::HashMap;
use pipebuilder_common::{
    app_directory,
    grpc::build::{builder_server::Builder, BuildResponse, CancelResponse, VersionBuildKey},
    grpc::{build::ListResponse, manifest::manifest_client::ManifestClient},
    remove_directory, Build, BuildStatus, LocalBuildContext, Register, VersionBuild,
};
use std::sync::Arc;
use tonic::{transport::Channel, Response};
use tracing::{error, info, warn};

pub struct BuilderService {
    lease_id: i64,
    register: Register,
    manifest_client: ManifestClient<Channel>,
    context: LocalBuildContext,
    // builds in progress, namespace/id/version -> join handle
    builds: Arc<HashMap<(String, String, u64), tokio::task::JoinHandle<()>>>,
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
            builds: Arc::new(HashMap::new()),
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
        let namespace = request.namespace;
        let id = request.id;
        let manifest_version = request.manifest_version;
        // lock build snapshot with manifest id
        // update latest build version
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let snapshot = match register
            .incr_build_snapshot(namespace.as_str(), id.as_str(), lease_id)
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
            namespace,
            id,
            manifest_version,
            manifest_client,
            build_version,
            build_context,
            target_platform,
        );
        // start build
        let lease_id = self.lease_id;
        let register = self.register.to_owned();
        let builds = self.builds.clone();
        start_build(lease_id, register, builds, build);
        Ok(Response::new(BuildResponse {
            version: build_version,
        }))
    }

    async fn cancel(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::build::CancelRequest>,
    ) -> Result<tonic::Response<pipebuilder_common::grpc::build::CancelResponse>, tonic::Status>
    {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        let version = request.build_version;
        let builds = self.builds.clone();
        let workspace = self.context.workspace.as_str();
        match cancel_local_build(builds, workspace, namespace.as_str(), id.as_str(), version) {
            Ok(succeed) => info!("cancel local build succeed: {}", succeed),
            Err(err) => {
                return Err(tonic::Status::internal(format!(
                    "cancel local build failed, error: '{:#?}'",
                    err
                )))
            }
        };
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        match cancel_version_build(
            &mut register,
            lease_id,
            namespace.as_str(),
            id.as_str(),
            version,
        )
        .await
        {
            Ok(_) => Ok(Response::new(CancelResponse {})),
            Err(err) => {
                return Err(tonic::Status::internal(format!(
                    "cancel version build failed, error: '{:#?}'",
                    err
                )))
            }
        }
    }

    async fn list(
        &self,
        _request: tonic::Request<pipebuilder_common::grpc::build::ListRequest>,
    ) -> Result<tonic::Response<pipebuilder_common::grpc::build::ListResponse>, tonic::Status> {
        let builds_ref = self.builds.pin();
        let keys = builds_ref
            .keys()
            .into_iter()
            .map(|(namespace, id, build_version)| VersionBuildKey {
                namespace: namespace.to_owned(),
                id: id.to_owned(),
                build_version: build_version.to_owned(),
            })
            .collect::<Vec<VersionBuildKey>>();
        Ok(Response::new(ListResponse { keys }))
    }
}

fn start_build(
    lease_id: i64,
    mut register: Register,
    builds: Arc<HashMap<(String, String, u64), tokio::task::JoinHandle<()>>>,
    mut build: Build,
) {
    let builds_clone = builds.clone();
    let key_tuple = build.get_build_key_tuple();
    let jh = tokio::spawn(async move {
        let mut status = BuildStatus::Pull;
        loop {
            // update build status in register
            match update(&mut register, lease_id, &build, status.clone(), None).await {
                Ok(()) => (),
                Err(err) => {
                    let (namespace, id, _, build_version) = build.get_build_meta();
                    error!(
                        "update build status failed for key: '{}/{}:{}', error: '{:#?}'",
                        namespace, id, build_version, err
                    );
                    break;
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
                    break;
                }
            };
            // continue next state or exit
            match next_status {
                Some(next_status) => status = next_status,
                None => break,
            }
        }
        let key_tuple = build.get_build_key_tuple();
        // remove local build
        builds_clone.pin().remove(&key_tuple);
    });
    // register local build
    builds.pin().insert(key_tuple, jh);
}

// update version build status
async fn update(
    register: &mut Register,
    lease_id: i64,
    build: &Build,
    status: BuildStatus,
    message: Option<String>,
) -> pipebuilder_common::Result<()> {
    let (namespace, id, _, build_version) = build.get_build_meta();
    let (builder_id, builder_address) = build.get_builder_meta();
    let now = Utc::now();
    let version_build = VersionBuild::new(
        status,
        now,
        builder_id.to_owned(),
        builder_address.to_owned(),
        message,
    );
    register
        .put_version_build(
            lease_id,
            namespace.as_str(),
            id.as_str(),
            build_version,
            version_build,
        )
        .await?;
    Ok(())
}

fn cancel_local_build(
    builds: Arc<HashMap<(String, String, u64), tokio::task::JoinHandle<()>>>,
    workspace: &str,
    namespace: &str,
    id: &str,
    version: u64,
) -> pipebuilder_common::Result<bool> {
    let key_tuple = (namespace.to_owned(), id.to_owned(), version);
    let builds_ref = builds.pin();
    match builds_ref.remove(&key_tuple) {
        Some(jh) => jh.abort(),
        None => {
            warn!("cancel non-extists build {}/{}/{}", namespace, id, version);
            return Ok(false);
        }
    };
    // cleanup local build workspace
    let path = app_directory(workspace, namespace, id, version);
    let succeed = remove_directory(path.as_str())?;
    Ok(succeed)
}

async fn cancel_version_build(
    register: &mut Register,
    lease_id: i64,
    namespace: &str,
    id: &str,
    version: u64,
) -> pipebuilder_common::Result<()> {
    let mut version_build = match register
        .get_version_build(lease_id, namespace, id, version)
        .await?
    {
        Some(version_build) => version_build,
        None => {
            warn!("cancel non-extists build {}/{}/{}", namespace, id, version);
            return Ok(());
        }
    };
    version_build.status = BuildStatus::Cancel;
    register
        .put_version_build(lease_id, namespace, id, version, version_build)
        .await?;
    Ok(())
}
