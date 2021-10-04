use crate::{
    errors::Result,
    utils::{
        app_build_log_path, app_build_release_path, app_build_target_path, app_directory,
        app_main_path, app_publish_path, app_restore_path, app_toml_manifest_path,
        build_get_manifest_request, cargo_build, cargo_fmt, cargo_init, copy_directory, copy_file,
        create_directory, move_directory, parse_toml, remove_directory, write_file, write_toml,
        TomlManifest,
    },
};
use chrono::{DateTime, Utc};
use pipegen::models::App;
use serde::{Deserialize, Serialize};
use tonic::transport::Channel;
use tracing::{error, info};

use crate::grpc::manifest::manifest_client::ManifestClient;

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

// Build state per (build_id, version)
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
    pub publish_directory: String,
}

impl LocalBuildContext {
    pub fn new(
        id: String,
        address: String,
        workspace: String,
        restore_directory: String,
        log_directory: String,
        publish_directory: String,
    ) -> Self {
        LocalBuildContext {
            id,
            address,
            workspace,
            restore_directory,
            log_directory,
            publish_directory,
        }
    }
}

// App build
pub struct Build {
    pub namespace: String,
    pub manifest_id: String,
    pub manifest_version: u64,
    pub manifest_client: ManifestClient<Channel>,
    pub build_version: u64,
    pub build_context: LocalBuildContext,
    pub target_platform: String,
    pub app: Option<App>,
}

impl Build {
    pub fn new(
        namespace: String,
        manifest_id: String,
        manifest_version: u64,
        manifest_client: ManifestClient<Channel>,
        build_version: u64,
        build_context: LocalBuildContext,
        target_platform: String,
    ) -> Self {
        Build {
            namespace,
            manifest_id,
            manifest_version,
            manifest_client,
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

    fn get_publish_directory(&self) -> &String {
        &self.build_context.publish_directory
    }

    pub fn get_build_meta(&self) -> (&String, &String, u64, u64) {
        let namespace = &self.namespace;
        let manifest_id = &self.manifest_id;
        let manifest_version = self.manifest_version;
        let build_version = self.build_version;
        (namespace, manifest_id, manifest_version, build_version)
    }

    pub fn get_build_key_tuple(&self) -> (String, String, u64) {
        (
            self.namespace.to_owned(),
            self.manifest_id.to_owned(),
            self.build_version,
        )
    }

    // run current status and return next status
    pub async fn run(&mut self, status: BuildStatus) -> Result<Option<BuildStatus>> {
        match status {
            BuildStatus::Pull => self.pull_manifest().await,
            BuildStatus::Validate => self.validate_manifest(),
            BuildStatus::Create => self.create_build_workspace(),
            BuildStatus::Generate => self.generate_app(),
            BuildStatus::Build => self.build_app(),
            BuildStatus::Publish => self.publish_app().await,
            BuildStatus::Store => self.store_app().await,
            BuildStatus::Succeed => self.succeed(),
            _ => unreachable!(),
        }
    }

    pub async fn pull_manifest(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, manifest_id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "pull manifest '{}/{}:({}, {})'",
            namespace, manifest_id, manifest_version, build_version
        );
        let request = build_get_manifest_request(
            namespace.to_owned(),
            manifest_id.to_owned(),
            manifest_version,
        );
        let response = self
            .manifest_client
            .get_manifest(request)
            .await?
            .into_inner();
        let buffer = response.buffer;
        let app = App::read_from_buffer(buffer.as_slice())?;
        self.app = Some(app);
        Ok(Some(BuildStatus::Validate))
    }

    pub fn validate_manifest(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, manifest_id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "validate manifest '{}/{}:({}, {})'",
            namespace, manifest_id, manifest_version, build_version
        );
        self.app.as_ref().expect("app not initialized").validate()?;
        Ok(Some(BuildStatus::Create))
    }

    pub fn create_build_workspace(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, manifest_id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "create build workspace for manifest '{}/{}:({}, {})'",
            namespace, manifest_id, manifest_version, build_version
        );
        let workspace = self.get_workspace().as_str();
        let restore_directory = self.get_restore_directory().as_str();
        let app_directory = app_directory(workspace, manifest_id, build_version);
        let app_restore_directory = app_restore_path(restore_directory, manifest_id, build_version);
        if !copy_directory(app_restore_directory.as_str(), app_directory.as_str())? {
            // cargo init
            create_directory(app_directory.as_str())?;
            cargo_init(app_directory.as_str())?;
        }
        Ok(Some(BuildStatus::Generate))
    }

    pub fn generate_app(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, manifest_id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "generate app for '{}/{}:({}, {})'",
            namespace, manifest_id, manifest_version, build_version
        );
        // update dependency Cargo.toml
        let workspace = self.get_workspace().as_str();
        let toml_path = app_toml_manifest_path(workspace, manifest_id, build_version);
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
        let main_path = app_main_path(workspace, manifest_id, build_version);
        write_file(main_path.as_str(), generated_code.as_bytes())?;
        // fmt code
        cargo_fmt(toml_path.as_str())?;
        Ok(Some(BuildStatus::Build))
    }

    pub fn build_app(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, manifest_id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "build app for '{}/{}:({}, {})'",
            namespace, manifest_id, manifest_version, build_version
        );
        // local build context
        let workspace = self.get_workspace().as_str();
        let log_directory = self.get_log_directory().as_str();
        // cargo build and stream log to file
        let target_platform = self.target_platform.as_str();
        let toml_path = app_toml_manifest_path(workspace, manifest_id, build_version);
        let log_path = app_build_log_path(log_directory, manifest_id, build_version);
        let target_path = app_build_target_path(workspace, manifest_id, build_version);
        cargo_build(
            toml_path.as_str(),
            target_platform,
            target_path.as_str(),
            log_path.as_str(),
        )?;
        Ok(Some(BuildStatus::Publish))
    }

    pub async fn publish_app(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, manifest_id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "publish app binary for '{}/{}:({}, {})'",
            namespace, manifest_id, manifest_version, build_version
        );
        // publish app binaries
        let workspace = self.get_workspace().as_str();
        let publish_directory = self.get_publish_directory().as_str();
        let release_path = app_build_release_path(workspace, manifest_id.as_str(), build_version);
        let publish_path = app_publish_path(publish_directory, manifest_id.as_str(), build_version);
        let size = copy_file(release_path.as_str(), publish_path.as_str())?;
        info!("published app binariy size: {} Mb", size / 1024 / 1024);
        Ok(Some(BuildStatus::Store))
    }

    pub async fn store_app(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, manifest_id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "store app for '{}/{}:({}, {})'",
            namespace, manifest_id, manifest_version, build_version
        );
        let workspace = self.get_workspace().as_str();
        let restore_directory = self.get_restore_directory().as_str();
        let app_directory = app_directory(workspace, manifest_id, build_version);
        let app_restore_directory = app_restore_path(restore_directory, manifest_id, build_version);
        // cleanup previous app build cache if any
        let _ = remove_directory(app_restore_directory.as_str())?;
        if !move_directory(app_directory.as_str(), app_restore_directory.as_str())? {
            error!(
                "store app from '{}' to '{}' failed",
                app_directory, app_restore_directory
            )
        }
        Ok(Some(BuildStatus::Succeed))
    }

    pub fn succeed(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, manifest_id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "build succeed for {}/{}:({}, {})",
            namespace, manifest_id, manifest_version, build_version
        );
        Ok(None)
    }
}
