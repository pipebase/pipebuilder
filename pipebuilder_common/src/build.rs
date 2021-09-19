use serde::Deserialize;

#[derive(Deserialize)]
pub enum VersionBuildStatus {
    // register version build
    Create,
    // pull manifest
    Pull,
    // validate manifest
    Validate,
    // initialize rust app
    // restore from previous build or create a new project
    Initialize,
    // generate rust code
    Generate,
    // cargo build
    Build,
    // store compiled results
    Store,
    // publish app binary
    Publish,
    // build failed
    Fail,
}

// Build state per (build_id, version)
#[derive(Deserialize)]
pub struct VersionBuild {
    pub status: VersionBuildStatus,
}

// Latest build state per manifest url
#[derive(Deserialize)]
pub struct BuildSnapshot {
    pub id: String,
    pub latest_version: u64,
}

// A build task
pub struct Build {
    pub manifest_url: String,
    pub id: String,
    pub version: u64,
}
