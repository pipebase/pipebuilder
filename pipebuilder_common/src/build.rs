use crate::{
    app_workspace,
    constants::{
        PATH_APP, PATH_APP_BUILD_LOG, PATH_APP_MAIN, PATH_APP_RELEASE_BINARY, PATH_APP_TARGET,
        PATH_APP_TOML_MANIFEST,
    },
    errors::Result,
    grpc::repository::{GetManifestRequest, PostAppRequest},
    read_file,
    utils::{
        app_build_log_directory, app_restore_directory, cargo_build, cargo_fmt, cargo_init,
        copy_directory, create_directory, move_directory, parse_toml, remove_directory, sub_path,
        write_file, write_toml, TomlManifest,
    },
};
use chrono::{DateTime, Utc};
use pipegen::models::App;
use serde::{Deserialize, Serialize};
use tonic::transport::Channel;
use tracing::info;

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
        status: BuildStatus,
        timestamp: DateTime<Utc>,
        builder_id: String,
        builder_address: String,
        message: Option<String>,
    ) -> Self {
        BuildMetadata {
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

// Latest build state per manifest id
#[derive(Default, Deserialize, Serialize)]
pub struct BuildSnapshot {
    pub latest_version: u64,
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
            "aarch64-unknown-linux-gnu"
                | "i686-pc-windows-gnu"
                | "i686-pc-windows-msvc"
                | "i686-unknown-linux-gnu"
                | "x86_64-apple-darwin"
                | "x86_64-pc-windows-gnu"
                | "x86_64-pc-windows-msvc"
                | "x86_64-unknown-linux-gnu"
        )
    }

    // read app build log
    pub async fn read_log(
        log_directory: &str,
        namespace: &str,
        id: &str,
        version: u64,
    ) -> Result<Vec<u8>> {
        let app_log_directory = app_build_log_directory(log_directory, namespace, id, version);
        let app_log_path = sub_path(app_log_directory.as_str(), PATH_APP_BUILD_LOG);
        read_file(app_log_path.as_str()).await
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

    pub fn get_build_meta(&self) -> (&String, &String, u64, u64) {
        let namespace = &self.namespace;
        let id = &self.id;
        let manifest_version = self.manifest_version;
        let build_version = self.build_version;
        (namespace, id, manifest_version, build_version)
    }

    pub fn get_build_key_tuple(&self) -> (String, String, u64) {
        (
            self.namespace.to_owned(),
            self.id.to_owned(),
            self.build_version,
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
        let (namespace, id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "pull manifest '{}/{}:({}, {})'",
            namespace, id, manifest_version, build_version
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
        let (namespace, id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "validate manifest '{}/{}:({}, {})'",
            namespace, id, manifest_version, build_version
        );
        self.app.as_ref().expect("app not initialized").validate()?;
        Ok(Some(BuildStatus::Create))
    }

    pub async fn create_build_workspace(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "create build workspace for manifest '{}/{}:({}, {})'",
            namespace, id, manifest_version, build_version
        );
        let workspace = self.get_workspace().as_str();
        let restore_directory = self.get_restore_directory().as_str();
        let namespace = self.namespace.as_str();
        // try restore app build workspace
        let app_workspace = app_workspace(workspace, namespace, id, build_version);
        create_directory(app_workspace.as_str()).await?;
        let target_platform = self.target_platform.as_str();
        let app_restore_directory =
            app_restore_directory(restore_directory, namespace, id, target_platform);
        let app_restore_path = sub_path(app_restore_directory.as_str(), PATH_APP);
        if !copy_directory(app_restore_path.clone(), app_workspace.clone()).await? {
            // cargo init
            let app_path = sub_path(app_workspace.as_str(), PATH_APP);
            create_directory(app_path.as_str()).await?;
            cargo_init(app_path.as_str()).await?;
        }
        Ok(Some(BuildStatus::Generate))
    }

    pub async fn generate_app(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "generate app for '{}/{}:({}, {})'",
            namespace, id, manifest_version, build_version
        );
        // update dependency Cargo.toml
        let workspace = self.get_workspace().as_str();
        let namespace = self.namespace.as_str();
        let app_workspace = app_workspace(workspace, namespace, id, build_version);
        let toml_path = sub_path(app_workspace.as_str(), PATH_APP_TOML_MANIFEST);
        let mut toml_manifest: TomlManifest = parse_toml(toml_path.as_str()).await?;
        toml_manifest.init();
        let app = self.app.as_ref().expect("app not initialized");
        let additionals = app.get_dependencies().clone();
        for additional in additionals {
            toml_manifest.add_dependency(additional.get_name(), additional.into());
        }
        write_toml(&toml_manifest, toml_path.as_str()).await?;
        // generate src/main.rs
        let generated_code = self.app.as_ref().expect("app not initialized").generate();
        let main_path = sub_path(app_workspace.as_str(), PATH_APP_MAIN);
        write_file(main_path.as_str(), generated_code.as_bytes()).await?;
        // fmt code
        cargo_fmt(toml_path.as_str()).await?;
        Ok(Some(BuildStatus::Build))
    }

    pub async fn build_app(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "build app for '{}/{}:({}, {})'",
            namespace, id, manifest_version, build_version
        );
        // local build context
        let workspace = self.get_workspace().as_str();
        let log_directory = self.get_log_directory().as_str();
        let namespace = self.namespace.as_str();
        // cargo build and stream log to file
        let app_workspace = app_workspace(workspace, namespace, id, build_version);
        let target_platform = self.target_platform.as_str();
        let toml_path = sub_path(app_workspace.as_str(), PATH_APP_TOML_MANIFEST);
        let target_path = sub_path(app_workspace.as_str(), PATH_APP_TARGET);
        // prepare log directory
        let log_directory = app_build_log_directory(log_directory, namespace, id, build_version);
        create_directory(log_directory.as_str()).await?;
        let log_path = sub_path(log_directory.as_str(), PATH_APP_BUILD_LOG);
        cargo_build(
            toml_path.as_str(),
            target_platform,
            target_path.as_str(),
            log_path.as_str(),
        )
        .await?;
        Ok(Some(BuildStatus::Publish))
    }

    // publish app binary
    pub async fn publish_app(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "publish app binary for '{}/{}:({}, {})'",
            namespace, id, manifest_version, build_version
        );
        // publish app binaries
        let workspace = self.get_workspace().as_str();
        let namespace = self.namespace.as_str();
        let app_workspace = app_workspace(workspace, namespace, id, build_version);
        let target_path = sub_path(app_workspace.as_str(), PATH_APP_TARGET);
        let target_platform = self.target_platform.as_str();
        let target_platform_release_binary =
            format!("{}/{}", target_platform, PATH_APP_RELEASE_BINARY);
        let release_path = sub_path(
            target_path.as_str(),
            target_platform_release_binary.as_str(),
        );
        let buffer = read_file(release_path.as_str()).await?;
        let request = self.build_post_app_request(buffer);
        let _ = self.repository_client.post_app(request).await?.into_inner();
        Ok(Some(BuildStatus::Store))
    }

    // store cargo project
    pub async fn store_app(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "store app for '{}/{}:({}, {})'",
            namespace, id, manifest_version, build_version
        );
        let workspace = self.get_workspace().as_str();
        let restore_directory = self.get_restore_directory().as_str();
        let namespace = self.namespace.as_str();
        let app_directory = app_workspace(workspace, namespace, id, build_version);
        let app_path = sub_path(app_directory.as_str(), PATH_APP);
        let target_platform = self.target_platform.as_str();
        let app_restore_directory =
            app_restore_directory(restore_directory, namespace, id, target_platform);
        let app_restore_path = sub_path(app_restore_directory.as_str(), PATH_APP);
        // cleanup previous app build cache if any
        // TODO: acquire lock avoid racing of two concurrent build
        let _ = remove_directory(app_restore_path.as_str()).await;
        create_directory(app_restore_path.as_str()).await?;
        move_directory(app_path.as_str(), app_restore_path.as_str()).await?;
        Ok(Some(BuildStatus::Succeed))
    }

    pub fn succeed(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "build succeed for {}/{}:({}, {})",
            namespace, id, manifest_version, build_version
        );
        Ok(None)
    }
}
