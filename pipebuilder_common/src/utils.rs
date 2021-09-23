use crate::grpc::manifest::GetManifestRequest;
use crate::{Error, Result};
use etcd_client::{Event, EventType};
use serde::de::DeserializeOwned;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use tracing::info;

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

pub fn parse_config<C>(file: File) -> Result<C>
where
    C: DeserializeOwned,
{
    let config = serde_yaml::from_reader::<std::fs::File, C>(file)?;
    Ok(config)
}

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

// etcd ops
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
