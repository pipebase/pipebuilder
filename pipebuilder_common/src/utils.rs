use crate::{
    errors::{cargo_error, Result},
    grpc::{build::builder_client::BuilderClient, node::node_client::NodeClient},
};
use chrono::{DateTime, TimeZone, Utc};
use etcd_client::{Event, EventType};
use filetime::FileTime;
use fnv::FnvHasher;
use fslock::{LockFile, ToOsStr};
use pipegen::models::Dependency;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsString,
    future::Future,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    pin::Pin,
};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::Command,
};
use tonic::transport::Channel;
use tracing::{info, warn};

// filesystem ops
pub async fn open_file<P>(path: P) -> Result<File>
where
    P: AsRef<std::path::Path>,
{
    let file = fs::File::open(path).await?;
    Ok(file)
}

pub async fn create_file<P>(path: P) -> Result<File>
where
    P: AsRef<std::path::Path>,
{
    let file = fs::File::create(path).await?;
    Ok(file)
}

pub async fn read_file<P>(path: P) -> Result<Vec<u8>>
where
    P: AsRef<std::path::Path>,
{
    let file = open_file(path).await?;
    let mut rdr = BufReader::new(file);
    let mut buffer: Vec<u8> = Vec::new();
    rdr.read_to_end(&mut buffer).await?;
    Ok(buffer)
}

pub async fn write_file<P>(path: P, buffer: &[u8]) -> Result<()>
where
    P: AsRef<std::path::Path>,
{
    let file = create_file(path).await?;
    let mut wrt = BufWriter::new(file);
    wrt.write_all(buffer).await?;
    wrt.flush().await?;
    Ok(())
}

pub fn open_lock_file<P>(path: &P) -> Result<LockFile>
where
    P: ToOsStr + ?Sized,
{
    let file = LockFile::open(path)?;
    Ok(file)
}

pub async fn create_directory<P>(path: P) -> Result<()>
where
    P: AsRef<std::path::Path>,
{
    fs::create_dir_all(path).await?;
    Ok(())
}

pub async fn reset_directory<P>(path: &P) -> Result<()>
where
    P: AsRef<std::path::Path>,
{
    remove_directory(path).await?;
    create_directory(path).await
}

pub async fn parse_config<C>(file: File) -> Result<C>
where
    C: DeserializeOwned,
{
    let file = file.into_std().await;
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
pub async fn remove_directory<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    fs::remove_dir_all(path).await?;
    Ok(())
}

pub fn copy_directory<P>(from: P, to: P) -> Pin<Box<dyn Future<Output = Result<bool>> + Send>>
where
    P: AsRef<Path> + Send + 'static,
{
    Box::pin(async move {
        let mut from_path = PathBuf::new();
        from_path.push(from);
        let mut to_path = PathBuf::new();
        to_path.push(to);
        if !from_path.exists() {
            warn!("copy from path '{}' does not exist", from_path.display());
            return Ok(false);
        }
        // extract last segment from and append to target
        let file_name = match from_path.file_name() {
            Some(file_name) => file_name,
            None => {
                warn!("file name not found in path '{}'.", from_path.display());
                return Ok(false);
            }
        };
        to_path.push(file_name);
        if from_path.is_file() {
            copy_file(&from_path, &to_path).await?;
            copy_file_times(&from_path, &to_path).await?;
            return Ok(true);
        }
        if !from_path.is_dir() {
            warn!(
                "copy from path '{}' is neither a file or directory",
                from_path.display()
            );
            return Ok(false);
        }
        create_directory(&to_path).await?;
        copy_file_times(&from_path, &to_path).await?;
        let mut entries = fs::read_dir(&from_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let from_entry_path = entry.path();
            copy_directory(from_entry_path, to_path.clone()).await?;
        }
        Ok(true)
    })
}

// move directory and return success flag
pub async fn move_directory<P>(from: P, to: P) -> Result<()>
where
    P: AsRef<Path>,
{
    fs::rename(from, to).await?;
    Ok(())
}

pub async fn copy_file<P>(from: P, to: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let _ = fs::copy(from, to).await?;
    Ok(())
}

pub async fn copy_file_times<P>(from: P, to: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let metadata = fs::metadata(from).await?;
    let atime = FileTime::from_last_access_time(&metadata);
    let mtime = FileTime::from_last_modification_time(&metadata);
    filetime::set_file_times(to, atime, mtime)?;
    Ok(())
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
pub async fn cargo_build(cargo_workdir: &str, target_platform: &str, log_path: &str) -> Result<()> {
    let log_file = fs::File::create(log_path).await?.into_std().await;
    let mut cmd = Command::new(cargo_binary());
    cmd.arg("build")
        .arg("--target")
        .arg(target_platform)
        .arg("--release");
    cmd.stderr(log_file);
    // change cwd as cargo workdir
    let current_dir = std::env::current_dir()?;
    std::env::set_current_dir(cargo_workdir)?;
    let code = cmd_status(cmd).await?;
    // restore previous workdir
    std::env::set_current_dir(current_dir)?;
    match code == 0 {
        true => Ok(()),
        false => Err(cargo_error("build", code, String::from("check build log"))),
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

pub async fn parse_toml<M, P>(toml_path: P) -> Result<M>
where
    M: DeserializeOwned,
    P: AsRef<Path>,
{
    let toml_string = fs::read_to_string(toml_path).await?;
    let toml_manifest = toml::from_str::<M>(toml_string.as_str())?;
    Ok(toml_manifest)
}

pub async fn write_toml<M, P>(object: &M, path: P) -> Result<()>
where
    M: Serialize,
    P: AsRef<Path>,
{
    let toml_string = toml::to_string(object)?;
    fs::write(path, toml_string).await?;
    Ok(())
}

// prost type conversion
pub fn prost_timestamp_to_datetime_utc(timestamp: prost_types::Timestamp) -> DateTime<Utc> {
    let secs = timestamp.seconds;
    let nsecs = timestamp.nanos as u32;
    Utc.timestamp(secs, nsecs)
}

pub fn datetime_utc_to_prost_timestamp(datetime: DateTime<Utc>) -> prost_types::Timestamp {
    let seconds = datetime.timestamp();
    let nanos = datetime.timestamp_subsec_nanos() as i32;
    prost_types::Timestamp { seconds, nanos }
}

#[cfg(test)]
mod tests {

    use crate::{copy_directory, read_file, remove_directory, Result};

    #[tokio::test]
    async fn test_copy_directory() -> Result<()> {
        let from = "resources/utils/files/from/app";
        let to = "resources/utils/files/to";
        let aloha = "resources/utils/files/to/app/file.txt";
        let hello = "resources/utils/files/to/app/src/file.txt";
        assert!(copy_directory(from, to).await.is_ok());
        let buffer = read_file(aloha).await?;
        let actual = String::from_utf8(buffer)?;
        assert_eq!("aloha", actual.as_str());
        let buffer = read_file(hello).await?;
        let actual = String::from_utf8(buffer)?;
        assert_eq!("hello", actual.as_str());
        remove_directory("resources/utils/files/to/app").await
    }
}
