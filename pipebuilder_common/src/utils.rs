use crate::{
    errors::{cargo_error, Error, Result},
    grpc::manifest::GetManifestRequest,
};
use etcd_client::{Event, EventType};
use pipegen::models::Dependency;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsString,
    fs::{self, File},
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    process::Command,
};
use tracing::info;

// filesystem ops
pub fn open_file<P>(path: P) -> Result<File>
where
    P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(path)?;
    Ok(file)
}

pub fn read_file<P>(path: P) -> Result<Vec<u8>>
where
    P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(path)?;
    let mut rdr = BufReader::new(file);
    let mut buffer: Vec<u8> = Vec::new();
    rdr.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn write_file<P>(path: P, buffer: &[u8]) -> Result<()>
where
    P: AsRef<std::path::Path>,
{
    let mut wrt = BufWriter::new(fs::File::create(path)?);
    wrt.write_all(buffer)?;
    wrt.flush()?;
    Ok(())
}

pub fn create_directory<P>(path: P) -> Result<()>
where
    P: AsRef<std::path::Path>,
{
    fs::create_dir_all(path)?;
    Ok(())
}

pub fn parse_config<C>(file: File) -> Result<C>
where
    C: DeserializeOwned,
{
    let config = serde_yaml::from_reader::<std::fs::File, C>(file)?;
    Ok(config)
}

pub fn app_directory(workspace: &str, manifest_id: &str, build_version: u64) -> String {
    format!("{}/{}/{}/app", workspace, manifest_id, build_version)
}

pub fn app_toml_manifest_path(workspace: &str, manifest_id: &str, build_version: u64) -> String {
    format!(
        "{}/{}/{}/app/Cargo.toml",
        workspace, manifest_id, build_version
    )
}

pub fn app_main_path(workspace: &str, manifest_id: &str, build_version: u64) -> String {
    format!(
        "{}/{}/{}/app/src/main.rs",
        workspace, manifest_id, build_version
    )
}

pub fn app_build_log_path(log_directory: &str, manifest_id: &str, build_version: u64) -> String {
    format!(
        "{}/{}/{}/build.log",
        log_directory, manifest_id, build_version
    )
}

// cmd ops
fn run_cmd(mut cmd: Command) -> Result<(i32, String)> {
    let output = cmd.output()?;
    match output.status.success() {
        true => {
            let stderr = String::from_utf8(output.stderr)?;
            Ok((0, stderr))
        }
        false => {
            let stderr = String::from_utf8(output.stderr)?;
            let err_code = output.status.code().unwrap_or(1);
            Ok((err_code, stderr))
        }
    }
}

// cargo ops
fn cargo_binary() -> OsString {
    match std::env::var_os("CARGO") {
        Some(cargo) => cargo,
        None => "cargo".to_owned().into(),
    }
}

pub fn cargo_init(path: &str) -> Result<()> {
    let mut cmd = Command::new(cargo_binary());
    cmd.arg("init").arg(path);
    let (code, out) = run_cmd(cmd)?;
    match code == 0 {
        true => Ok(()),
        false => Err(cargo_error("init", code, out)),
    }
}

pub fn cargo_fmt(manifest_path: &str) -> Result<()> {
    let mut cmd = Command::new(cargo_binary());
    cmd.arg("fmt").arg("--manifest-path").arg(manifest_path);
    let (code, out) = run_cmd(cmd)?;
    match code == 0 {
        true => Ok(()),
        false => Err(cargo_error("fmt", code, out)),
    }
}

// target platform: https://doc.rust-lang.org/cargo/commands/cargo-build.html#compilation-options
pub fn cargo_build(
    manifest_path: &str,
    target_platform: &str,
    target_directory: &str,
    log_path: &str,
) -> Result<()> {
    let mut cmd = Command::new(cargo_binary());
    cmd.arg("build")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--target")
        .arg(target_platform)
        .arg("--target-dir")
        .arg(target_directory)
        .arg("--release");
    cmd.arg("&>").arg(log_path);
    let (code, _) = run_cmd(cmd)?;
    match code == 0 {
        true => Ok(()),
        false => Err(cargo_error(
            "build",
            code,
            String::from("checkout build log"),
        )),
    }
}

// etcd ops
pub fn log_event(event: &Event) -> Result<()> {
    if let Some(kv) = event.kv() {
        let event = match event.event_type() {
            EventType::Delete => "delete",
            EventType::Put => "put",
        };
        info!("[event] type: {}, key: {}", event, kv.key_str()?,);
    }
    Ok(())
}

pub fn deserialize_event<T>(event: &Event) -> Result<Option<(EventType, String, T)>>
where
    T: DeserializeOwned,
{
    if let Some(kv) = event.kv() {
        let key = kv.key_str()?;
        let value = kv.value();
        let value = serde_json::from_slice::<T>(value)?;
        return Ok(Some((event.event_type(), key.to_owned(), value)));
    }
    Ok(None)
}

pub fn prefix_namespace_id_key(prefix: &str, namespace: &str, id: &str) -> String {
    format!("{}/{}/{}", prefix, namespace, id)
}

pub fn prefix_namespace_id_version_key(
    prefix: &str,
    namespace: &str,
    id: &str,
    version: u64,
) -> String {
    format!("{}/{}/{}/{}", prefix, namespace, id, version)
}

pub fn resource_namespace(prefix: &str, namespace: &str) -> String {
    format!("{}/{}", prefix, namespace)
}

// remove /resource/namespace and return id/<suffix> given a key
pub fn remove_resource_namespace<'a>(key: &'a str, prefix: &str, namespace: &str) -> &'a str {
    let pattern = format!("{}/{}", prefix, namespace);
    key.strip_prefix(pattern.as_str()).expect(&format!(
        "key '{}' not start with '/{}/{}'",
        key, prefix, namespace
    ))
}

// rpc status
pub fn internal_error(error: Error) -> tonic::Status {
    tonic::Status::internal(format!("{:#?}", error))
}

pub fn not_found(message: &str) -> tonic::Status {
    tonic::Status::not_found(message)
}

// rpc request
pub fn build_get_manifest_request(
    namespace: String,
    id: String,
    version: u64,
) -> GetManifestRequest {
    GetManifestRequest {
        namespace,
        id,
        version,
    }
}

// App cargo.toml
#[derive(Deserialize, Serialize, Debug)]
pub struct TomlProject {
    name: String,
    version: String,
    authors: Option<Vec<String>>,
    edition: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct TomlDependency {
    version: Option<String>,
    path: Option<String>,
    git: Option<String>,
    branch: Option<String>,
    tag: Option<String>,
    features: Option<Vec<String>>,
    package: Option<String>,
}

impl From<Dependency> for TomlDependency {
    fn from(pd: Dependency) -> Self {
        TomlDependency {
            version: pd.get_version(),
            path: pd.get_path(),
            git: pd.get_git(),
            branch: pd.get_branch(),
            tag: pd.get_tag(),
            features: pd.get_features(),
            package: pd.get_package(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TomlWorkspace {}

impl TomlWorkspace {
    pub fn new() -> Self {
        TomlWorkspace {}
    }
}

impl Default for TomlWorkspace {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct TomlManifest {
    package: Option<TomlProject>,
    dependencies: Option<HashMap<String, TomlDependency>>,
    workspace: Option<TomlWorkspace>,
}

impl TomlManifest {
    pub fn init(&mut self) {
        self.dependencies = Some(HashMap::new());
        self.workspace = Some(TomlWorkspace::new());
    }

    pub fn add_dependency(&mut self, name: String, dependency: TomlDependency) {
        let dependencies = self.dependencies.as_mut().unwrap();
        dependencies.insert(name, dependency);
    }
}

pub fn parse_toml<M, P>(toml_path: P) -> Result<M>
where
    M: DeserializeOwned,
    P: AsRef<Path>,
{
    let toml_string = fs::read_to_string(toml_path)?;
    let toml_manifest = toml::from_str::<M>(toml_string.as_str())?;
    Ok(toml_manifest)
}

pub fn write_toml<M, P>(object: &M, path: P) -> Result<()>
where
    M: Serialize,
    P: AsRef<Path>,
{
    let toml_string = toml::to_string(object)?;
    fs::write(path, toml_string)?;
    Ok(())
}
