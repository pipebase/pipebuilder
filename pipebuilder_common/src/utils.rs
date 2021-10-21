use crate::{
    errors::{cargo_error, Result},
    grpc::{build::builder_client::BuilderClient, node::node_client::NodeClient},
};
use etcd_client::{Event, EventType};
use fnv::FnvHasher;
use pipegen::models::Dependency;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsString,
    fs::{self, File},
    hash::{Hash, Hasher},
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};
use tokio::process::Command;
use tonic::transport::Channel;
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

// app build workspace
pub fn app_workspace(workspace: &str, namespace: &str, id: &str, build_version: u64) -> String {
    format!("{}/{}/{}/{}", workspace, namespace, id, build_version)
}

pub fn app_build_log_directory(
    log_directory: &str,
    namespace: &str,
    id: &str,
    build_version: u64,
) -> String {
    format!("{}/{}/{}/{}", log_directory, namespace, id, build_version)
}

pub fn app_restore_directory(
    restore_directory: &str,
    namespace: &str,
    id: &str,
    target_platform: &str,
) -> String {
    format!(
        "{}/{}/{}/{}",
        restore_directory, namespace, id, target_platform
    )
}

pub fn sub_path(parent_directory: &str, path: &str) -> String {
    format!("{}/{}", parent_directory, path)
}

// remove directory and return success flag
pub async fn remove_directory(path: &str) -> Result<bool> {
    let mut cmd = Command::new("rm");
    cmd.arg("-r").arg(path);
    let (code, _) = cmd_status_output(cmd).await?;
    Ok(code == 0)
}

// copy directory and return success flag
pub async fn copy_directory(src: &str, dst: &str) -> Result<bool> {
    let mut cmd = Command::new("cp");
    cmd.arg("-r").arg(src).arg(dst);
    let (code, _) = cmd_status_output(cmd).await?;
    Ok(code == 0)
}

// move directory and return success flag
pub async fn move_directory(src: &str, dst: &str) -> Result<bool> {
    let mut cmd = Command::new("mv");
    cmd.arg(src).arg(dst);
    let (code, _) = cmd_status_output(cmd).await?;
    Ok(code == 0)
}

pub fn copy_file(from: &str, to: &str) -> Result<u64> {
    let size = std::fs::copy(from, to)?;
    Ok(size)
}

// run cmd and collect status and output
async fn cmd_status_output(mut cmd: Command) -> Result<(i32, String)> {
    let output = cmd.output().await?;
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

// run cmd and collect status
async fn cmd_status(mut cmd: Command) -> Result<i32> {
    let status = cmd.status().await?;
    match status.success() {
        true => Ok(0),
        false => Ok(status.code().unwrap_or(1)),
    }
}

// cargo ops
fn cargo_binary() -> OsString {
    match std::env::var_os("CARGO") {
        Some(cargo) => cargo,
        None => "cargo".to_owned().into(),
    }
}

pub async fn cargo_init(path: &str) -> Result<()> {
    let mut cmd = Command::new(cargo_binary());
    cmd.arg("init").arg(path);
    let (code, out) = cmd_status_output(cmd).await?;
    match code == 0 {
        true => Ok(()),
        false => Err(cargo_error("init", code, out)),
    }
}

pub async fn cargo_fmt(manifest_path: &str) -> Result<()> {
    let mut cmd = Command::new(cargo_binary());
    cmd.arg("fmt").arg("--manifest-path").arg(manifest_path);
    let (code, out) = cmd_status_output(cmd).await?;
    match code == 0 {
        true => Ok(()),
        false => Err(cargo_error("fmt", code, out)),
    }
}

// target platform: https://doc.rust-lang.org/cargo/commands/cargo-build.html#compilation-options
pub async fn cargo_build(
    manifest_path: &str,
    target_platform: &str,
    target_directory: &str,
    log_path: &str,
) -> Result<()> {
    let log_file = fs::File::create(log_path)?;
    let mut cmd = Command::new(cargo_binary());
    cmd.arg("build")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--target")
        .arg(target_platform)
        .arg("--target-dir")
        .arg(target_directory)
        .arg("--release");
    cmd.stderr(log_file);
    let code = cmd_status(cmd).await?;
    match code == 0 {
        true => Ok(()),
        false => {
            Err(cargo_error("build", code, String::from("check build log")))
            // Err(cargo_error("build", code, message))
        }
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

pub fn deserialize_event<T>(event: &Event) -> Result<Option<(EventType, String, Option<T>)>>
where
    T: DeserializeOwned,
{
    if let Some(kv) = event.kv() {
        let key = kv.key_str()?;
        let value = kv.value();
        // in case delete, value is empty
        let value = match value.is_empty() {
            false => Some(serde_json::from_slice::<T>(value)?),
            true => None,
        };
        return Ok(Some((event.event_type(), key.to_owned(), value)));
    }
    Ok(None)
}

pub fn resource_namespace_id(resource: &str, namespace: &str, id: &str) -> String {
    format!("{}/{}/{}", resource, namespace, id)
}

pub fn resource_namespace_id_version(
    resource: &str,
    namespace: &str,
    id: &str,
    version: u64,
) -> String {
    format!("{}/{}/{}/{}", resource, namespace, id, version)
}

pub fn resource_namespace(resource: &str, namespace: &str) -> String {
    format!("{}/{}", resource, namespace)
}

// remove /resource/namespace/ and return id/<suffix> given a key
pub fn remove_resource_namespace<'a>(key: &'a str, resource: &str, namespace: &str) -> &'a str {
    let pattern = format!("{}/{}/", resource, namespace);
    key.strip_prefix(pattern.as_str())
        .unwrap_or_else(|| panic!("key '{}' not start with '/{}/{}'", key, resource, namespace))
}

// rpc
pub async fn build_builder_client(protocol: &str, address: &str) -> Result<BuilderClient<Channel>> {
    let client = BuilderClient::connect(format!("{}://{}", protocol, address)).await?;
    Ok(client)
}

pub async fn build_node_client(protocol: &str, address: &str) -> Result<NodeClient<Channel>> {
    let client = NodeClient::connect(format!("{}://{}", protocol, address)).await?;
    Ok(client)
}

// hash
fn fnv1a<T>(t: &T) -> u64
where
    T: Hash,
{
    let mut hasher = FnvHasher::default();
    t.hash(&mut hasher);
    hasher.finish()
}

pub fn hash_distance<T>(t0: &T, t1: &T) -> u64
where
    T: Hash,
{
    let h0 = fnv1a(t0);
    let h1 = fnv1a(t1);
    if h0 > h1 {
        return h0 - h1;
    }
    h1 - h0
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
