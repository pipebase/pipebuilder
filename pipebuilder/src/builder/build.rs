use chrono::Utc;
use flurry::HashMap;
use pipebuilder_common::{
    self, build_error, datetime_utc_to_prost_timestamp,
    grpc::{
        build::{
            builder_server::Builder, BuildCacheMetadata as RpcBuildCacheMetadata, BuildMetadataKey,
            BuildResponse, CancelBuildResponse, DeleteBuildCacheResponse, GetBuildLogResponse,
            ScanBuildCacheResponse, ScanBuildResponse,
        },
        repository::repository_client::RepositoryClient,
    },
    remove_directory, reset_directory, Build, BuildCacheMetadata, BuildMetadata, BuildSnapshot,
    BuildStatus, LocalBuildContext, PathBuilder, Register, Snapshot, PATH_APP,
};
use std::sync::Arc;
use tonic::{transport::Channel, Response};
use tracing::{error, info, warn};

pub struct BuildManager {
    lease_id: i64,
    register: Register,
    repository_client: RepositoryClient<Channel>,
    context: LocalBuildContext,
    // builds in progress, (namespace, id, version) -> build thread join handle
    builds: Arc<HashMap<(String, String, u64), tokio::task::JoinHandle<()>>>,
    // pre-build caches, (namespace, id, target_platform)
    caches: Arc<HashMap<(String, String, String), BuildCacheMetadata>>,
}

#[derive(Default)]
pub struct BuildManagerBuilder {
    lease_id: Option<i64>,
    register: Option<Register>,
    repository_client: Option<RepositoryClient<Channel>>,
    context: Option<LocalBuildContext>,
}

impl BuildManagerBuilder {
    pub fn lease_id(mut self, lease_id: i64) -> Self {
        self.lease_id = Some(lease_id);
        self
    }

    pub fn register(mut self, register: Register) -> Self {
        self.register = Some(register);
        self
    }

    pub fn repository_client(mut self, repository_client: RepositoryClient<Channel>) -> Self {
        self.repository_client = Some(repository_client);
        self
    }

    pub fn context(mut self, context: LocalBuildContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn build(self) -> BuildManager {
        BuildManager {
            lease_id: self.lease_id.expect("lease id undefined"),
            register: self.register.expect("register undefined"),
            repository_client: self.repository_client.expect("repository client undefined"),
            context: self.context.expect("local build context undefined"),
            builds: Arc::new(HashMap::new()),
            caches: Arc::new(HashMap::new()),
        }
    }
}

impl BuildManager {
    pub fn builder() -> BuildManagerBuilder {
        BuildManagerBuilder::default()
    }

    pub async fn init(&self, reset: bool) -> pipebuilder_common::Result<()> {
        let workspace = &self.context.workspace;
        let restore_directory = &self.context.restore_directory;
        let log_directory = &self.context.log_directory;
        if reset {
            info!(path = workspace.as_str(), "reset workspace directory");
            reset_directory(workspace).await?;
            info!(path = restore_directory.as_str(), "reset restore directory");
            reset_directory(restore_directory).await?;
            info!(path = log_directory.as_str(), "reset log directory");
            reset_directory(log_directory).await?;
        }
        Ok(())
    }

    pub async fn build(
        &self,
        namespace: &str,
        id: &str,
        manifest_version: u64,
        target_platform: &str,
    ) -> pipebuilder_common::Result<u64> {
        // lock build snapshot with manifest id
        // update latest build version
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        let snapshot = match register
            .update_snapshot_resource::<BuildSnapshot>(namespace, id, lease_id)
            .await {
                Ok((_, snapshot)) => snapshot,
                Err(err) => return Err(build_error(String::from("update snapshot"), format!("update snapshot failed for (namespace = {}, id = {}, manifest_version = {}, error: {:#?})", namespace, id, manifest_version, err))),
            };
        // prepare build contexts
        let build_version = snapshot.get_version();
        let manifest_client = self.repository_client.clone();
        let build_context = self.context.to_owned();
        // start build
        info!(
            namespace = namespace,
            id = id,
            manifest_version = manifest_version,
            build_version = build_version,
            "start build"
        );
        let build = Build::new(
            namespace.to_owned(),
            id.to_owned(),
            manifest_version,
            manifest_client,
            build_version,
            build_context,
            target_platform.to_owned(),
        );
        let lease_id = self.lease_id;
        let register = self.register.to_owned();
        let builds = self.builds.clone();
        let caches = self.caches.clone();
        Self::start_build(lease_id, register, builds, build, caches);
        Ok(build_version)
    }

    pub async fn cancel_build(
        &self,
        namespace: &str,
        id: &str,
        build_version: u64,
    ) -> pipebuilder_common::Result<()> {
        let builds = self.builds.clone();
        let workspace = self.context.workspace.as_str();
        // stop local build thread
        if !Self::cancel_local_build(builds, namespace, id, build_version) {
            return Err(build_error(
                String::from("cancel build"),
                format!(
                    "local build not found for (namespace = {}, id = {}, build_version = {})",
                    namespace, id, build_version
                ),
            ));
        }
        // cleanup local build workspace
        let app_path = PathBuilder::default()
            .push(workspace)
            .push(namespace)
            .push(id)
            .push(build_version.to_string())
            .push(PATH_APP)
            .build();
        if remove_directory(app_path.as_path()).await.is_err() {
            error!(
                namespace = namespace,
                id = id,
                build_version = build_version,
                "clean app directory failed"
            )
        };
        let mut register = self.register.clone();
        let lease_id = self.lease_id;
        // update metadata
        match Self::cancel_build_metadata(&mut register, lease_id, namespace, id, build_version).await {
            Ok(_) => Ok(()),
            Err(err) => Err(build_error(String::from("update build metadata"), format!("update build metadata failed for (namespace = {}, id = {}, version = {}), error: {:#?}", namespace, id, build_version, err))),
        }
    }

    pub fn scan_build(&self) -> Vec<BuildMetadataKey> {
        let builds_ref = self.builds.pin();
        builds_ref
            .keys()
            .into_iter()
            .map(|(namespace, id, build_version)| BuildMetadataKey {
                namespace: namespace.to_owned(),
                id: id.to_owned(),
                version: build_version.to_owned(),
            })
            .collect::<Vec<BuildMetadataKey>>()
    }

    pub async fn get_build_log(
        &self,
        namespace: &str,
        id: &str,
        build_version: u64,
    ) -> pipebuilder_common::Result<Vec<u8>> {
        let log_directory = self.context.log_directory.as_str();
        match Build::read_log(log_directory, namespace, id, build_version).await {
            Ok(buffer) => Ok(buffer),
            Err(err) => Err(build_error(String::from("get build log"), format!("get build log failed for (namespace = {}, id = {}, version = {}), error: {:#?}", namespace, id, build_version, err)))
        }
    }

    pub async fn delete_build_cache(
        &self,
        namespace: &str,
        id: &str,
        target_platform: &str,
    ) -> pipebuilder_common::Result<()> {
        let restore_directory = self.context.restore_directory.as_str();
        match Build::delete_build_cache(
            restore_directory,
            namespace,
            id,
            target_platform,
        )
        .await
        {
            Ok(_) => {
                // cleanup build cache key set
                let caches = self.caches.pin();
                caches.remove(&(namespace.to_owned(), id.to_owned(), target_platform.to_owned()));
                Ok(())
            }
            Err(err) => {
                Err(build_error(
                    String::from("delete build cache"),
                    format!("delete build cache failed for (namespace = {}, id = {}, target_platform = {},), error: '{:#?}'",
                            namespace, id, target_platform, err)
                ))
            }
        }
    }

    pub fn scan_build_cache(&self) -> Vec<RpcBuildCacheMetadata> {
        let caches = self.caches.pin();
        caches
            .into_iter()
            .map(
                |((namespace, id, target_platform), metadata)| RpcBuildCacheMetadata {
                    namespace: namespace.to_owned(),
                    id: id.to_owned(),
                    target_platform: target_platform.to_owned(),
                    timestamp: Some(datetime_utc_to_prost_timestamp(metadata.get_timestamp())),
                },
            )
            .collect::<Vec<RpcBuildCacheMetadata>>()
    }

    fn start_build(
        lease_id: i64,
        mut register: Register,
        builds: Arc<HashMap<(String, String, u64), tokio::task::JoinHandle<()>>>,
        mut build: Build,
        caches: Arc<HashMap<(String, String, String), BuildCacheMetadata>>,
    ) {
        let builds_clone = builds.clone();
        let key_tuple = build.get_build_key_tuple();
        let jh = tokio::spawn(async move {
            let mut status = BuildStatus::Pull;
            loop {
                // update build status in register
                match Self::update(&mut register, lease_id, &build, status.clone(), None).await {
                    Ok(()) => (),
                    Err(err) => {
                        let (namespace, id, manifest_version, build_version, target_platform) =
                            build.get_build_meta();
                        error!(
                            namespace = namespace.as_str(),
                            id = id.as_str(),
                            manifest_version = manifest_version,
                            build_version = build_version,
                            target_platform = target_platform.as_str(),
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
                        let (namespace, id, manifest_version, build_version, target_platform) =
                            build.get_build_meta();
                        error!(
                            namespace = namespace.as_str(),
                            id = id.as_str(),
                            manifest_version = manifest_version,
                            build_version = build_version,
                            target_platform = target_platform.as_str(),
                            "run build fail, status: '{}', error: '{:#?}'",
                            status.to_string(),
                            err
                        );
                        let _ = Self::update(
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
            let build_key_tuple = build.get_build_key_tuple();
            // remove local build
            builds_clone.pin().remove(&build_key_tuple);
            // update build cache key set if build succeed
            if matches!(status, BuildStatus::Succeed) {
                let build_cache_key_tuple = build.get_build_cache_key_tuple();
                let build_cache_metadata = BuildCacheMetadata::new();
                caches
                    .pin()
                    .insert(build_cache_key_tuple, build_cache_metadata);
            }
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
        let (namespace, id, _, build_version, target_platform) = build.get_build_meta();
        let (builder_id, builder_address) = build.get_builder_meta();
        let now = Utc::now();
        let build_metadata = BuildMetadata::new(
            target_platform.to_owned(),
            status,
            now,
            builder_id.to_owned(),
            builder_address.to_owned(),
            message,
        );
        register
            .put_resource(
                Some(namespace.as_str()),
                id.as_str(),
                Some(build_version),
                build_metadata,
                lease_id,
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

    async fn cancel_build_metadata(
        register: &mut Register,
        lease_id: i64,
        namespace: &str,
        id: &str,
        version: u64,
    ) -> pipebuilder_common::Result<()> {
        let mut build_metadata = match register
            .get_resource::<BuildMetadata>(Some(namespace), id, Some(version), lease_id)
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
        build_metadata.status = BuildStatus::Cancel;
        register
            .put_resource(Some(namespace), id, Some(version), build_metadata, lease_id)
            .await?;
        Ok(())
    }
}

pub struct BuilderService {
    manager: BuildManager,
}

impl BuilderService {
    pub fn new(manager: BuildManager) -> Self {
        BuilderService { manager }
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
        let target_platform = request.target_platform;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            manifest_version = manifest_version,
            target_platform = target_platform.as_str(),
            "build"
        );
        match self
            .manager
            .build(
                namespace.as_str(),
                id.as_str(),
                manifest_version,
                target_platform.as_str(),
            )
            .await
        {
            Ok(build_version) => Ok(Response::new(BuildResponse {
                version: build_version,
            })),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    manifest_version = manifest_version,
                    target_platform = target_platform.as_str(),
                    "trigger build failed, error: {:#?}",
                    err
                );
                Err(tonic::Status::internal(format!("{:#?}", err)))
            }
        }
    }

    async fn cancel_build(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::build::CancelBuildRequest>,
    ) -> Result<tonic::Response<pipebuilder_common::grpc::build::CancelBuildResponse>, tonic::Status>
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
        match self
            .manager
            .cancel_build(namespace.as_str(), id.as_str(), version)
            .await
        {
            Ok(_) => Ok(Response::new(CancelBuildResponse {})),
            Err(err) => Err(tonic::Status::internal(format!(
                "cancel version build failed, error: '{:#?}'",
                err
            ))),
        }
    }

    async fn scan_build(
        &self,
        _request: tonic::Request<pipebuilder_common::grpc::build::ScanBuildRequest>,
    ) -> Result<tonic::Response<pipebuilder_common::grpc::build::ScanBuildResponse>, tonic::Status>
    {
        info!("scan local build");
        let builds = self.manager.scan_build();
        Ok(Response::new(ScanBuildResponse { builds }))
    }

    async fn get_build_log(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::build::GetBuildLogRequest>,
    ) -> Result<tonic::Response<pipebuilder_common::grpc::build::GetBuildLogResponse>, tonic::Status>
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
        match self
            .manager
            .get_build_log(namespace.as_str(), id.as_str(), version)
            .await
        {
            Ok(buffer) => Ok(Response::new(GetBuildLogResponse { buffer })),
            Err(err) => Err(tonic::Status::not_found(format!(
                "build log not found, error: '{}'",
                err
            ))),
        }
    }

    async fn delete_build_cache(
        &self,
        request: tonic::Request<pipebuilder_common::grpc::build::DeleteBuildCacheRequest>,
    ) -> Result<
        tonic::Response<pipebuilder_common::grpc::build::DeleteBuildCacheResponse>,
        tonic::Status,
    > {
        let request = request.into_inner();
        let namespace = request.namespace;
        let id = request.id;
        let target_platform = request.target_platform;
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            target_platform = target_platform.as_str(),
            "delete build cache"
        );
        match self
            .manager
            .delete_build_cache(namespace.as_str(), id.as_str(), target_platform.as_str())
            .await
        {
            Ok(_) => Ok(Response::new(DeleteBuildCacheResponse {})),
            Err(err) => {
                error!(
                    namespace = namespace.as_str(),
                    id = id.as_str(),
                    target_platform = target_platform.as_str(),
                    "delete build cache failed, error: '{:#?}'",
                    err
                );
                Err(tonic::Status::internal(format!(
                    "delete build cache failed, error: '{:#?}'",
                    err
                )))
            }
        }
    }

    async fn scan_build_cache(
        &self,
        _request: tonic::Request<pipebuilder_common::grpc::build::ScanBuildCacheRequest>,
    ) -> Result<
        tonic::Response<pipebuilder_common::grpc::build::ScanBuildCacheResponse>,
        tonic::Status,
    > {
        let caches = self.manager.scan_build_cache();
        Ok(Response::new(ScanBuildCacheResponse { caches }))
    }
}
