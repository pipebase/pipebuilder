use crate::{
    errors::{cargo_error, Error, Result},
    grpc::manifest::GetManifestRequest,
};
use etcd_client::{Event, EventType};
use serde::de::DeserializeOwned;
use std::{
    ffi::OsString,
    fs::{self, File},
    io::{BufReader, BufWriter, Read, Write},
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
    fs::create_dir(path)?;
    Ok(())
}

pub fn parse_config<C>(file: File) -> Result<C>
where
    C: DeserializeOwned,
{
    let config = serde_yaml::from_reader::<std::fs::File, C>(file)?;
    Ok(config)
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
pub fn cargo_build(manifest_path: &str, target_platform: &str, target_directory: &str, log_path: &str) -> Result<()> {
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

pub fn prefix_id_key(prefix: &str, id: &str) -> String {
    format!("{}/{}", prefix, id)
}

pub fn prefix_id_version_key(prefix: &str, id: &str, version: u64) -> String {
    format!("{}/{}/{}", prefix, id, version)
}

// rpc status
pub fn internal_error(error: Error) -> tonic::Status {
    tonic::Status::internal(format!("{:#?}", error))
}

pub fn not_found(message: &str) -> tonic::Status {
    tonic::Status::not_found(message)
}

// rpc request
pub fn build_get_manifest_request(id: String) -> GetManifestRequest {
    GetManifestRequest { id }
}
