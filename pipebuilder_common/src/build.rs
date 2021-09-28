use crate::{
    errors::Result,
    utils::{
        app_build_log_path, app_directory, app_main_path, app_toml_manifest_path,
        build_get_manifest_request, cargo_build, cargo_fmt, cargo_init, create_directory,
        parse_toml, write_file, write_toml, TomlManifest,
    },
};
use chrono::{DateTime, Utc};
use pipegen::models::App;
use serde::{Deserialize, Serialize};
use tonic::transport::Channel;
use tracing::info;

use crate::grpc::manifest::manifest_client::ManifestClient;

#[derive(Deserialize, Serialize, Clone)]
pub enum BuildStatus {
    // pull manifest
    Pull,
    // validate manifest
    Validate,
    // create build workspace
    Create,
    // restore previous compilation
    Restore,
    // generate rust code
    Generate,
    // cargo build
    Build,
    // store compiled results
    Store,
    // publish app binary
    Publish,
    // succeed all steps
    Succeed,
    // build failed
    Fail,
    // build interrupt due to node maintenance / deployment
    Interrupt,
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
    id: String,
    // builder external_address
    address: String,
    workspace: String,
    target_directory: String,
    log_directory: String,
}

impl LocalBuildContext {
    pub fn new(
        id: String,
        address: String,
        workspace: String,
        target_directory: String,
        log_directory: String,
    ) -> Self {
        LocalBuildContext {
            id,
            address,
            workspace,
            target_directory,
            log_directory,
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

    fn get_target_directory(&self) -> &String {
        &self.build_context.target_directory
    }

    pub fn get_build_meta(&self) -> (&String, &String, u64, u64) {
        let namespace = &self.namespace;
        let manifest_id = &self.manifest_id;
        let manifest_version = self.manifest_version;
        let build_version = self.build_version;
        (namespace, manifest_id, manifest_version, build_version)
    }

    // run current status and return next status
    pub async fn run(&mut self, status: BuildStatus) -> Result<Option<BuildStatus>> {
        match status {
            BuildStatus::Pull => self.pull_manifest().await,
            BuildStatus::Create => self.create_build(),
            BuildStatus::Validate => self.validate_manifest(),
            BuildStatus::Restore => self.restore_compilation().await,
            BuildStatus::Generate => self.generate_app(),
            BuildStatus::Build => self.build_app(),
            BuildStatus::Store => self.store_compilation().await,
            BuildStatus::Publish => self.publish_app().await,
            BuildStatus::Succeed => self.succeed(),
            _ => unreachable!(),
        }
    }

    pub async fn pull_manifest(&mut self) -> Result<Option<BuildStatus>> {
        let namespace = self.namespace.to_owned();
        let manifest_id = self.manifest_id.to_owned();
        let manifest_version = self.manifest_version;
        let request = build_get_manifest_request(namespace, manifest_id, manifest_version);
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
            "validate manifest {}/{}:({}, {})",
            namespace, manifest_id, manifest_version, build_version
        );
        self.app.as_ref().expect("app not initialized").validate()?;
        Ok(Some(BuildStatus::Create))
    }

    pub fn create_build(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, manifest_id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "create build workspace for manifest {}/{}:({}, {})",
            namespace, manifest_id, manifest_version, build_version
        );
        let workspace = self.get_workspace().as_str();
        let manifest_id = self.manifest_id.as_str();
        let build_version = self.build_version;
        let app_directory = app_directory(workspace, manifest_id, build_version);
        // cargo init
        create_directory(app_directory.as_str())?;
        cargo_init(app_directory.as_str())?;
        Ok(Some(BuildStatus::Restore))
    }

    pub async fn restore_compilation(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, manifest_id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "restore compilation for manifest {}/{}:({}, {})",
            namespace, manifest_id, manifest_version, build_version
        );
        // restore previous compilation if any
        Ok(Some(BuildStatus::Generate))
    }

    pub fn generate_app(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, manifest_id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "generate app for {}/{}:({}, {})",
            namespace, manifest_id, manifest_version, build_version
        );
        // update dependency Cargo.toml
        let workspace = self.get_workspace().as_str();
        let manifest_id = self.manifest_id.as_str();
        let build_version = self.build_version;
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
            "build app for {}/{}:({}, {})",
            namespace, manifest_id, manifest_version, build_version
        );
        let workspace = self.get_workspace().as_str();
        let manifest_id = self.manifest_id.as_str();
        let log_directory = self.get_log_directory().as_str();
        let build_version = self.build_version;
        // cargo build and stream log to file
        let target_platform = self.target_platform.as_str();
        let target_directory = self.get_target_directory().as_str();
        let toml_path = app_toml_manifest_path(workspace, manifest_id, build_version);
        let log_path = app_build_log_path(log_directory, manifest_id, build_version);
        cargo_build(
            toml_path.as_str(),
            target_platform,
            target_directory,
            log_path.as_str(),
        )?;
        Ok(Some(BuildStatus::Store))
    }

    pub async fn store_compilation(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, manifest_id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "store compilation for {}/{}:({}, {})",
            namespace, manifest_id, manifest_version, build_version
        );
        // store target folder
        Ok(Some(BuildStatus::Publish))
    }

    pub async fn publish_app(&mut self) -> Result<Option<BuildStatus>> {
        let (namespace, manifest_id, manifest_version, build_version) = self.get_build_meta();
        info!(
            "publish app binaries for {}/{}:({}, {})",
            namespace, manifest_id, manifest_version, build_version
        );
        // publish app binaries
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
