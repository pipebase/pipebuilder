use chrono::Utc;
use flurry::HashMap;
use pipebuilder_common::{
    app_workspace,
    grpc::{
        build::{
            builder_server::Builder, BuildMetadataKey, BuildResponse, CancelResponse,
            GetLogResponse, ScanResponse,
        },
        repository::repository_client::RepositoryClient,
    },
    remove_directory, sub_path, Build, BuildMetadata, BuildStatus, LocalBuildContext, Register,
    PATH_APP,
};
use std::sync::Arc;
use tonic::{transport::Channel, Response};
use tracing::{error, info, warn};

pub struct BuilderService {
    lease_id: i64,
    register: Register,
    repository_client: RepositoryClient<Channel>,
    context: LocalBuildContext,
    // builds in progress, namespace/id/version -> join handle
    builds: Arc<HashMap<(String, String, u64), tokio::task::JoinHandle<()>>>,
}

impl BuilderService {
    pub fn new(
        lease_id: i64,
        register: Register,
        repository_client: RepositoryClient<Channel>,
        context: LocalBuildContext,
    ) -> Self {
        BuilderService {
            lease_id,
            register,
            repository_client,
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
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            manifest_version = manifest_version,
            "build"
        );
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
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    manifest_version = manifest_version,
                    "trigger build failed, error: {:#?}",
                    err
                );
                return Err(tonic::Status::internal(format!("{:#?}", err)));
            }
        };
        // prepare build contexts
        let build_version = snapshot.latest_version;
        let manifest_client = self.repository_client.clone();
        let build_context = self.context.to_owned();
        let target_platform = request.target_platform;
        // start build
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            manifest_version = manifest_version,
            build_version = build_version,
            "start build"
        );
        let build = Build::new(
            namespace,
            id,
            manifest_version,
            manifest_client,
            build_version,
            build_context,
            target_platform,
        );
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
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            build_version = version,
            "cancel build"
        );
        let builds = self.builds.clone();
        let workspace = self.context.workspace.as_str();
        // stop local build thread
        if !cancel_local_build(builds, namespace.as_str(), id.as_str(), version) {
            return Err(tonic::Status::invalid_argument(format!(
                "local build not found for (namespace = {}, id = {}, build_version = {})",
                namespace, id, version
            )));
        }
        // cleanup local build workspace
        let app_directory = app_workspace(workspace, namespace.as_str(), id.as_str(), version);
        let app_path = sub_path(app_directory.as_str(), PATH_APP);
        if remove_directory(app_path.as_str()).await.is_err() {
            error!(
                namespace = namespace.as_str(),
                id = id.as_str(),
                build_version = version,
                "clean app directory failed"
            )
        };
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        // update metadata
        match cancel_build(
            &mut register,
            lease_id,
            namespace.as_str(),
            id.as_str(),
            version,
        )
        .await
        {
            Ok(_) => Ok(Response::new(CancelResponse {})),
            Err(err) => Err(tonic::Status::internal(format!(
                "cancel version build failed, error: '{:#?}'",
                err
            ))),
        }
    }

    async fn scan(
        &self,
        _request: tonic::Request<pipebuilder_common::grpc::build::ScanRequest>,
    ) -> Result<tonic::Response<pipebuilder_common::grpc::build::ScanResponse>, tonic::Status> {
        info!("scan local build");
        let builds_ref = self.builds.pin();
        let builds = builds_ref
            .keys()
            .into_iter()
            .map(|(namespace, id, build_version)| BuildMetadataKey {
                namespace: namespace.to_owned(),
                id: id.to_owned(),
                version: build_version.to_owned(),
            })
            .collect::<Vec<BuildMetadataKey>>();
        Ok(Response::new(ScanResponse { builds }))
    }

    async fn get_log(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::build::GetLogRequest>,
    ) -> Result<tonic::Response<pipebuilder_common::grpc::build::GetLogResponse>, tonic::Status>
    {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        let version = request.build_version;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            build_version = version,
            "get build log"
        );
        let log_directory = self.context.log_directory.as_str();
        match Build::read_log(log_directory, namespace.as_str(), id.as_str(), version).await {
            Ok(buffer) => Ok(Response::new(GetLogResponse { buffer })),
            Err(err) => Err(tonic::Status::not_found(format!(
                "build log for (namespace = {}, id = {}, build_version = {}) not found, error: '{}'",
                namespace, id, version, err
            ))),
        }
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
                    let (namespace, id, manifest_version, build_version) = build.get_build_meta();
                    error!(
                        namespace = namespace.as_str(),
                        id = id.as_str(),
                        manifest_version = manifest_version,
                        build_version = build_version,
                        "update build status fail, error: '{:#?}'",
                        err
                    );
                    break;
                }
            };
            // run current build state
            let result = build.run(status.clone()).await;
            let next_status = match result {
                Ok(next_status) => next_status,
                Err(err) => {
                    let (namespace, id, manifest_version, build_version) = build.get_build_meta();
                    error!(
                        namespace = namespace.as_str(),
                        id = id.as_str(),
                        manifest_version = manifest_version,
                        build_version = build_version,
                        "run build fail, status: '{}', error: '{:#?}'",
                        status.to_string(),
                        err
                    );
                    let _ = update(
                        &mut register,
                        lease_id,
                        &build,
                        BuildStatus::Fail,
                        Some(format!("{}", err)),
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
    let target_platform = build.get_target_platform();
    let now = Utc::now();
    let version_build = BuildMetadata::new(
        target_platform.to_owned(),
        status,
        now,
        builder_id.to_owned(),
        builder_address.to_owned(),
        message,
    );
    register
        .put_build_metadata(
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
    namespace: &str,
    id: &str,
    version: u64,
) -> bool {
    let key_tuple = (namespace.to_owned(), id.to_owned(), version);
    let builds_ref = builds.pin();
    match builds_ref.remove(&key_tuple) {
        Some(jh) => {
            jh.abort();
            true
        }
        None => {
            warn!(
                namespace = namespace,
                id = id,
                build_version = version,
                "cancel non-exists build"
            );
            false
        }
    }
}

async fn cancel_build(
    register: &mut Register,
    lease_id: i64,
    namespace: &str,
    id: &str,
    version: u64,
) -> pipebuilder_common::Result<()> {
    let mut version_build = match register
        .get_build_metadata(lease_id, namespace, id, version)
        .await?
    {
        Some(version_build) => version_build,
        None => {
            warn!(
                namespace = namespace,
                id = id,
                build_version = version,
                "cancel non-extists build"
            );
            return Ok(());
        }
    };
    version_build.status = BuildStatus::Cancel;
    register
        .put_build_metadata(lease_id, namespace, id, version, version_build)
        .await?;
    Ok(())
}
