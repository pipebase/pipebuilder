use crate::Result;
use etcd_client::{Event, EventType};
use serde::de::DeserializeOwned;
use std::fs::File;
use std::io::{BufReader, Read};
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
        info!(
            "[event] type: {}, key: {}",
            event,
            kv.key_str()?,
        );
    }
    Ok(())
}

pub fn deserialize_event<T>(event: &Event) -> Result<Option<(EventType, String, T)>> 
where
    T: DeserializeOwned
{
    if let Some(kv) = event.kv() {
        let key = kv.key_str()?;
        let value = kv.value();
        let value = serde_json::from_slice::<T>(value)?;
        return Ok(Some((event.event_type(), key.to_owned(), value)))
    }
    Ok(None)
}
