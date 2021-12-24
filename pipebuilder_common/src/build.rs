use crate::{
    constants::{
        PATH_APP, PATH_APP_BUILD_LOG, PATH_APP_LOCK, PATH_APP_MAIN, PATH_APP_RELEASE_BINARY,
        PATH_APP_TARGET, PATH_APP_TOML_MANIFEST,
    },
    errors::Result,
    grpc::repository::{GetManifestRequest, PostAppRequest},
    open_lock_file, read_file,
    utils::{
        cargo_build, cargo_fmt, cargo_init, copy_directory, create_directory, move_directory,
        parse_toml, remove_directory, write_file, write_toml, PathBuilder, TomlManifest,
    },
    Resource, ResourceType, Snapshot,
};
use chrono::{DateTime, Utc};
use pipegen::models::App;
use serde::{Deserialize, Serialize};
use tonic::transport::Channel;
use tracing::{info, warn};

use crate::grpc::repository::repository_client::RepositoryClient;

#[derive(Deserialize, Serialize, Clone)]
pub enum BuildStatus {
    // pull manifest
    Pull,
    // validate manifest
    Validate,
    // create or restore build workspace
    Create,
    // generate rust code
    Generate,
    // cargo build
    Build,
    // publish app binary
    Publish,
    // store compiled results
    Store,
    // succeed all steps
    Succeed,
    // build failed
    Fail,
    // build cancelled due to node maintenance / deployment or cli
    Cancel,
}

impl ToString for BuildStatus {
    fn to_string(&self) -> String {
        let status_text = match self {
            Self::Pull => "Pull",
            Self::Validate => "Validate",
            Self::Create => "Create",
            Self::Generate => "Generate",
            Self::Build => "Build",
            Self::Publish => "Publish",
            Self::Store => "Store",
            Self::Succeed => "Succeed",
            Self::Fail => "Fail",
            Self::Cancel => "Cancel",
        };
        String::from(status_text)
    }
}

// Build state per (build_id, version), persist in registry
#[derive(Deserialize, Serialize)]
pub struct BuildMetadata {
    // target platform
    pub target_platform: String,
    // build status
    pub status: BuildStatus,
    // timestamp
    pub timestamp: DateTime<Utc>,
    // builder id
    pub builder_id: String,
    // builder external adress
    pub builder_address: String,
    // message
    pub message: Option<String>,
}

impl BuildMetadata {
    pub fn new(
        target_platform: String,
        status: BuildStatus,
        timestamp: DateTime<Utc>,
        builder_id: String,
        builder_address: String,
        message: Option<String>,
    ) -> Self {
        BuildMetadata {
            target_platform,
            status,
            timestamp,
            builder_id,
            builder_address,
            message,
        }
    }

    pub fn is_stopped(&self) -> bool {
        matches!(
            self.status,
            BuildStatus::Cancel | BuildStatus::Fail | BuildStatus::Succeed
        )
    }
}

impl Resource for BuildMetadata {
    fn ty() -> ResourceType {
        ResourceType::BuildMetadata
    }
}

pub struct BuildCacheMetadata {
    pub timestamp: DateTime<Utc>,
}

impl BuildCacheMetadata {
    pub fn new() -> Self {
        BuildCacheMetadata {
            timestamp: Utc::now(),
        }
    }

    pub fn get_timestamp(&self) -> DateTime<Utc> {
        self.timestamp.to_owned()
    }
}

impl Default for BuildCacheMetadata {
    fn default() -> Self {
        Self::new()
    }
}

// Latest build state per manifest id
#[derive(Default, Deserialize, Serialize)]
pub struct BuildSnapshot {
    pub latest_version: u64,
}

impl Snapshot for BuildSnapshot {
    fn incr_version(&mut self) {
        self.latest_version += 1
    }
}

impl Resource for BuildSnapshot {
    fn ty() -> ResourceType {
        ResourceType::BuildSnapshot
    }
}

#[derive(Default)]
pub struct LocalBuildContextBuilder {
    // builder id
    id: Option<String>,
    // builder external_address
    address: Option<String>,
    workspace: Option<String>,
    restore_directory: Option<String>,
    log_directory: Option<String>,
}

impl LocalBuildContextBuilder {
    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn address(mut self, address: String) -> Self {
        self.address = Some(address);
        self
    }

    pub fn workspace(mut self, workspace: String) -> Self {
        self.workspace = Some(workspace);
        self
    }

    pub fn restore_directory(mut self, restore_directory: String) -> Self {
        self.restore_directory = Some(restore_directory);
        self
    }

    pub fn log_directory(mut self, log_directory: String) -> Self {
        self.log_directory = Some(log_directory);
        self
    }

    pub fn build(self) -> LocalBuildContext {
        LocalBuildContext {
            id: self.id.expect("builder id undefined"),
            address: self.address.expect("builder external address undefined"),
            workspace: self.workspace.expect("workspace directory undefined"),
            restore_directory: self.restore_directory.expect("restore directory undefined"),
            log_directory: self.log_directory.expect("log directory undefined"),
        }
    }
}

// build context shared by all local builds
#[derive(Clone)]
pub struct LocalBuildContext {
    // builder id
    pub id: String,
    // builder external_address
    pub address: String,
    pub workspace: String,
    pub restore_directory: String,
    pub log_directory: String,
}

impl LocalBuildContext {
    pub fn new(
        id: String,
        address: String,
        workspace: String,
        restore_directory: String,
        log_directory: String,
    ) -> Self {
        LocalBuildContext {
            id,
            address,
            workspace,
            restore_directory,
            log_directory,
        }
    }
}

// App build
pub struct Build {
    pub namespace: String,
    pub id: String,
    pub manifest_version: u64,
    pub repository_client: RepositoryClient<Channel>,
    pub build_version: u64,
    pub build_context: LocalBuildContext,
    // https://doc.rust-lang.org/nightly/rustc/platform-support.html
    pub target_platform: String,
    pub app: Option<App>,
}

impl Build {
    pub fn is_target_platform_support(target_platform: &str) -> bool {
        matches!(
            target_platform,
            "aarch64-unknown-linux-gnu" | "x86_64-apple-darwin" | "x86_64-unknown-linux-gnu"
        )
    }

    // read app build log
    pub async fn read_log(
        log_directory: &str,
        namespace: &str,
        id: &str,
        version: u64,
    ) -> Result<Vec<u8>> {
        let app_log_path = PathBuilder::default()
            .push(log_directory)
            .push(namespace)
            .push(id)
            .push(version.to_string())
            .push(PATH_APP_BUILD_LOG)
            .build();
        read_file(app_log_path.as_path()).await
    }

    pub fn new(
        namespace: String,
        id: String,
        manifest_version: u64,
        repository_client: RepositoryClient<Channel>,
        build_version: u64,
        build_context: LocalBuildContext,
        target_platform: String,
    ) -> Self {
        Build {
            namespace,
            id,
            manifest_version,
            repository_client,
            build_version,
            build_context,
            target_platform,
            app: None,
        }
    }

    // (id, address)
    pub fn get_builder_meta(&self) -> (&String, &String) {
        (&self.build_context.id, &self.build_context.address)
    }

    fn get_workspace(&self) -> &String {
        &self.build_context.workspace
    }

    fn get_log_directory(&self) -> &String {
        &self.build_context.log_directory
    }

    fn get_restore_directory(&self) -> &String {
        &self.build_context.restore_directory
    }

    pub fn get_build_meta(&self) -> (&String, &String, u64, u64, &String) {
        let namespace = &self.namespace;
        let id = &self.id;
        let manifest_version = self.manifest_version;
        let build_version = self.build_version;
        let target_platform = &self.target_platform;
        (
            namespace,
            id,
            manifest_version,
            build_version,
            target_platform,
        )
    }

    pub fn get_build_key_tuple(&self) -> (String, String, u64) {
        (
            self.namespace.to_owned(),
            self.id.to_owned(),
            self.build_version,
        )
    }

    pub fn get_build_cache_key_tuple(&self) -> (String, String, String) {
        (
            self.namespace.to_owned(),
            self.id.to_owned(),
            self.target_platform.to_owned(),
        )
    }

    // run current status and return next status
    pub async fn run(&mut self, status: BuildStatus) -> Result<Option<BuildStatus>> {
        match status {
            BuildStatus::Pull => self.pull_manifest().await,
            BuildStatus::Validate => self.validate_manifest(),
            BuildStatus::Create => self.create_build_workspace().await,
            BuildStatus::Generate => self.generate_app().await,
            BuildStatus::Build => self.build_app().await,
            BuildStatus::Publish => self.publish_app().await,
            BuildStatus::Store => self.store_app().await,
            BuildStatus::Succeed => self.succeed(),
            _ => unreachable!(),
        }
    }

    fn build_get_manifest_request(&self) -> GetManifestRequest {
        let namespace = self.namespace.to_owned();
        let id = self.id.to_owned();
        let version = self.manifest_version;
        GetManifestRequest {
            namespace,
            id,
            version,
        }
    }

    pub fn build_post_app_request(&self, buffer: Vec<u8>) -> PostAppRequest {
        let namespace = self.namespace.to_owned();
        let id = self.id.to_owned();
        let version = self.build_version;
        PostAppRequest {
            namespace,
            id,
            version,
            buffer,
        }
    }

    pub async fn pull_manifest(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, id, manifest_version, build_version, target_platform) =
            self.get_build_meta();
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            manifest_version = manifest_version,
            build_version = build_version,
            target_platform = target_platform.as_str(),
            "pull manifest"
        );
        let request = self.build_get_manifest_request();
        let response = self
            .repository_client
            .get_manifest(request)
            .await?
            .into_inner();
        let buffer = response.buffer;
        let app = App::read_from_buffer(buffer.as_slice())?;
        self.app = Some(app);
        Ok(Some(BuildStatus::Validate))
    }

    pub fn validate_manifest(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, id, manifest_version, build_version, target_platform) =
            self.get_build_meta();
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            manifest_version = manifest_version,
            build_version = build_version,
            target_platform = target_platform.as_str(),
            "validate manifest"
        );
        self.app.as_ref().expect("app not initialized").validate()?;
        Ok(Some(BuildStatus::Create))
    }

    pub async fn create_build_workspace(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, id, manifest_version, build_version, target_platform) =
            self.get_build_meta();
        let workspace = self.get_workspace().as_str();
        let restore_directory = self.get_restore_directory().as_str();
        let namespace = namespace.as_str();
        let id = id.as_str();
        let target_platform = target_platform.as_str();
        info!(
            namespace = namespace,
            id = id,
            manifest_version = manifest_version,
            build_version = build_version,
            target_platform = target_platform,
            "create build workspace"
        );
        // try restore app build workspace
        let app_workspace = PathBuilder::default()
            .push(workspace)
            .push(namespace)
            .push(id)
            .push(build_version.to_string())
            .build();
        create_directory(app_workspace.as_path()).await?;
        let app_restore_directory = PathBuilder::default()
            .push(restore_directory)
            .push(namespace)
            .push(id)
            .push(target_platform)
            .build();
        // create restore directory / lock file if not exists
        create_directory(app_restore_directory.as_path()).await?;
        let app_restore_path = PathBuilder::clone_from(&app_restore_directory)
            .push(PATH_APP)
            .build();
        let app_restore_lock_path = PathBuilder::clone_from(&app_restore_directory)
            .push(PATH_APP_LOCK)
            .build();
        let mut app_restore_lock_file = open_lock_file(app_restore_lock_path.as_path())?;
        if app_restore_lock_file.try_lock()? {
            // try restore from compiled app
            if copy_directory(app_restore_path.clone(), app_workspace.clone()).await? {
                app_restore_lock_file.unlock()?;
                info!(
                    namespace = namespace,
                    id = id,
                    manifest_version = manifest_version,
                    build_version = build_version,
                    "restore app succeed"
                );
                return Ok(Some(BuildStatus::Generate));
            }
            app_restore_lock_file.unlock()?;
        }
        info!(
            namespace = namespace,
            id = id,
            manifest_version = manifest_version,
            build_version = build_version,
            "can not restore app, cargo init ..."
        );
        // can not restore from compiled app, cargo init app
        // cargo init
        let app_path = PathBuilder::clone_from(&app_workspace)
            .push(PATH_APP)
            .build();
        create_directory(app_path.as_path()).await?;
        cargo_init(app_path.as_os_str()).await?;
        Ok(Some(BuildStatus::Generate))
    }

    pub async fn generate_app(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, id, manifest_version, build_version, target_platform) =
            self.get_build_meta();
        // update dependency Cargo.toml
        let workspace = self.get_workspace().as_str();
        let namespace = namespace.as_str();
        let id = id.as_str();
        let target_platform = target_platform.as_str();
        info!(
            namespace = namespace,
            id = id,
            manifest_version = manifest_version,
            build_version = build_version,
            target_platform = target_platform,
            "generate app"
        );
        let app_workspace = PathBuilder::default()
            .push(workspace)
            .push(namespace)
            .push(id)
            .push(build_version.to_string())
            .build();
        let toml_path = PathBuilder::clone_from(&app_workspace)
            .push(PATH_APP_TOML_MANIFEST)
            .build();
        let mut toml_manifest: TomlManifest = parse_toml(toml_path.as_path()).await?;
        toml_manifest.init();
        let app = self.app.as_ref().expect("app not initialized");
        let additionals = app.get_dependencies().clone();
        for additional in additionals {
            toml_manifest.add_dependency(additional.get_name(), additional.into());
        }
        write_toml(&toml_manifest, toml_path.as_path()).await?;
        // generate src/main.rs
        let generated_code = self.app.as_ref().expect("app not initialized").generate();
        let main_path = PathBuilder::clone_from(&app_workspace)
            .push(PATH_APP_MAIN)
            .build();
        write_file(main_path.as_path(), generated_code.as_bytes()).await?;
        // fmt code
        cargo_fmt(toml_path.as_os_str()).await?;
        Ok(Some(BuildStatus::Build))
    }

    pub async fn build_app(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, id, manifest_version, build_version, target_platform) =
            self.get_build_meta();
        // local build context
        let workspace = self.get_workspace().as_str();
        let log_directory = self.get_log_directory().as_str();
        let namespace = namespace.as_str();
        let id = id.as_str();
        let target_platform = target_platform.as_str();
        info!(
            namespace = namespace,
            id = id,
            manifest_version = manifest_version,
            build_version = build_version,
            target_platform = target_platform,
            "build app"
        );
        // cargo build and stream log to file
        let cargo_workdir = PathBuilder::default()
            .push(workspace)
            .push(namespace)
            .push(id)
            .push(build_version.to_string())
            .push(PATH_APP)
            .build();
        // prepare log directory
        let log_directory = PathBuilder::default()
            .push(log_directory)
            .push(namespace)
            .push(id)
            .push(build_version.to_string())
            .build();
        create_directory(log_directory.as_path()).await?;
        let log_path = PathBuilder::clone_from(&log_directory)
            .push(PATH_APP_BUILD_LOG)
            .build();
        cargo_build(cargo_workdir.as_path(), target_platform, log_path.as_path()).await?;
        Ok(Some(BuildStatus::Publish))
    }

    // publish app binary
    pub async fn publish_app(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, id, manifest_version, build_version, target_platform) =
            self.get_build_meta();
        // publish app binaries
        let workspace = self.get_workspace().as_str();
        let namespace = namespace.as_str();
        let id = id.as_str();
        let target_platform = target_platform.as_str();
        info!(
            namespace = namespace,
            id = id,
            manifest_version = manifest_version,
            build_version = build_version,
            target_platform = target_platform,
            "publish app binary"
        );
        let app_workspace = PathBuilder::default()
            .push(workspace)
            .push(namespace)
            .push(id)
            .push(build_version.to_string())
            .build();
        let target_path = PathBuilder::clone_from(&app_workspace)
            .push(PATH_APP_TARGET)
            .build();
        let release_path = PathBuilder::clone_from(&target_path)
            .push(target_platform)
            .push(PATH_APP_RELEASE_BINARY)
            .build();
        let buffer = read_file(release_path.as_path()).await?;
        let request = self.build_post_app_request(buffer);
        let _ = self.repository_client.post_app(request).await?.into_inner();
        Ok(Some(BuildStatus::Store))
    }

    // store cargo project
    pub async fn store_app(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, id, manifest_version, build_version, target_platform) =
            self.get_build_meta();
        let workspace = self.get_workspace().as_str();
        let restore_directory = self.get_restore_directory().as_str();
        let namespace = namespace.as_str();
        let id = id.as_str();
        let target_platform = target_platform.as_str();
        info!(
            namespace = namespace,
            id = id,
            manifest_version = manifest_version,
            build_version = build_version,
            target_platform = target_platform,
            "store app"
        );

        let app_path = PathBuilder::default()
            .push(workspace)
            .push(namespace)
            .push(id)
            .push(build_version.to_string())
            .push(PATH_APP)
            .build();
        let app_restore_directory = PathBuilder::default()
            .push(restore_directory)
            .push(namespace)
            .push(id)
            .push(target_platform)
            .build();
        // create restore directory / lock file if not exists
        create_directory(app_restore_directory.as_path()).await?;
        let app_restore_path = PathBuilder::clone_from(&app_restore_directory)
            .push(PATH_APP)
            .build();
        let app_restore_lock_path = PathBuilder::clone_from(&app_restore_directory)
            .push(PATH_APP_LOCK)
            .build();
        let mut app_restore_lock_file = open_lock_file(app_restore_lock_path.as_path())?;
        // lock file before store compiled app - avoid corruption due to race
        if !app_restore_lock_file.try_lock()? {
            warn!(
                namespace = namespace,
                id = id,
                manifest_version = manifest_version,
                build_version = build_version,
                target_platform = target_platform,
                "lock file failed, skip app store ...",
            );
            return Ok(Some(BuildStatus::Succeed));
        }
        // do store compiled app
        let r = Self::do_store_app(&app_path, &app_restore_path).await;
        // unlock file regardless of store succeed or fail
        app_restore_lock_file.unlock()?;
        match r {
            Ok(_) => Ok(Some(BuildStatus::Succeed)),
            Err(err) => Err(err),
        }
    }

    async fn do_store_app(
        app_path: &std::path::Path,
        app_restore_path: &std::path::Path,
    ) -> Result<()> {
        // cleanup previous app build cache if any
        let _ = remove_directory(app_restore_path).await;
        create_directory(app_restore_path).await?;
        move_directory(app_path, app_restore_path).await
    }

    pub fn succeed(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, id, manifest_version, build_version, target_platform) =
            self.get_build_meta();
        info!(
            namespace = namespace.as_str(),
            id = id.as_str(),
            manifest_version = manifest_version,
            build_version = build_version,
            target_platform = target_platform.as_str(),
            "build succeed"
        );
        Ok(None)
    }

    pub async fn delete_build_cache(
        restore_directory: &str,
        namespace: &str,
        id: &str,
        target_platform: &str,
    ) -> Result<()> {
        let app_restore_directory = PathBuilder::default()
            .push(restore_directory)
            .push(namespace)
            .push(id)
            .push(target_platform)
            .build();
        let app_restore_path = PathBuilder::clone_from(&app_restore_directory)
            .push(PATH_APP)
            .build();
        let app_restore_lock_path = PathBuilder::clone_from(&app_restore_directory)
            .push(PATH_APP_LOCK)
            .build();
        let mut app_restore_lock_file = open_lock_file(app_restore_lock_path.as_path())?;
        if app_restore_lock_file.try_lock()? {
            remove_directory(app_restore_path.as_path()).await?;
            app_restore_lock_file.unlock()?;
        }
        Ok(())
    }
}
