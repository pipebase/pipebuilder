use crate::{
    app_directory,
    errors::Result,
    read_file,
    utils::{
        app_build_log_directory, app_build_log_path, app_build_release_path, app_build_target_path,
        app_main_path, app_path, app_restore_directory, app_restore_path, app_toml_manifest_path,
        build_get_manifest_request, build_post_app_request, cargo_build, cargo_fmt, cargo_init,
        copy_directory, create_directory, move_directory, parse_toml, remove_directory, write_file,
        write_toml, TomlManifest,
    },
};
use chrono::{DateTime, Utc};
use pipegen::models::App;
use serde::{Deserialize, Serialize};
use tonic::transport::Channel;
use tracing::{error, info};

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
pub struct VersionBuild {
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

impl VersionBuild {
    pub fn new(
        status: BuildStatus,
        timestamp: DateTime<Utc>,
        builder_id: String,
        builder_address: String,
        message: Option<String>,
    ) -> Self {
        VersionBuild {
            status,
            timestamp,
            builder_id,
            builder_address,
            message,
        }
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

    pub async fn pull_manifest(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "pull manifest '{}/{}:({}, {})'",
            namespace, id, manifest_version, build_version
        );
        let request =
            build_get_manifest_request(namespace.to_owned(), id.to_owned(), manifest_version);
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
        let app_directory = app_directory(workspace, namespace, id, build_version);
        create_directory(app_directory.as_str())?;
        let target_platform = self.target_platform.as_str();
        let app_restore_path = app_restore_path(restore_directory, namespace, id, target_platform);
        if !copy_directory(app_restore_path.as_str(), app_directory.as_str()).await? {
            // cargo init
            let app_path = app_path(workspace, namespace, id, build_version);
            create_directory(app_path.as_str())?;
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
        let toml_path = app_toml_manifest_path(workspace, namespace, id, build_version);
        let mut toml_manifest: TomlManifest = parse_toml(toml_path.as_str())?;
        toml_manifest.init();
        let app = self.app.as_ref().expect("app not initialized");
        let additionals = app.get_dependencies().clone();
        for additional in additionals {
            toml_manifest.add_dependency(additional.get_name(), additional.into());
        }
        write_toml(&toml_manifest, toml_path.as_str())?;
        // generate src/main.rs
        let generated_code = self.app.as_ref().expect("app not initialized").generate();
        let main_path = app_main_path(workspace, namespace, id, build_version);
        write_file(main_path.as_str(), generated_code.as_bytes())?;
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
        let target_platform = self.target_platform.as_str();
        let toml_path = app_toml_manifest_path(workspace, namespace, id, build_version);
        let target_path = app_build_target_path(workspace, namespace, id, build_version);
        // prepare log directory
        let log_directory = app_build_log_directory(log_directory, namespace, id, build_version);
        create_directory(log_directory.as_str())?;
        let log_path = app_build_log_path(log_directory.as_str());
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
        let target_platform = self.target_platform.as_str();
        let release_path = app_build_release_path(
            workspace,
            namespace,
            id.as_str(),
            build_version,
            target_platform,
        );
        let buffer = read_file(release_path.as_str())?;
        let request =
            build_post_app_request(namespace.to_owned(), id.to_owned(), build_version, buffer);
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
        let app_path = app_path(workspace, namespace, id, build_version);
        let target_platform = self.target_platform.as_str();
        let app_restore_directory =
            app_restore_directory(restore_directory, namespace, id, target_platform);
        let app_restore_path = app_restore_path(restore_directory, namespace, id, target_platform);
        // cleanup previous app build cache if any
        let _ = remove_directory(app_restore_path.as_str()).await?;
        create_directory(app_restore_directory.as_str())?;
        if !move_directory(app_path.as_str(), app_restore_directory.as_str()).await? {
            error!(
                "store app from '{}' to '{}' failed",
                app_path, app_restore_directory
            )
        }
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
