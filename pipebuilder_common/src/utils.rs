use crate::Result;
use serde::de::DeserializeOwned;
use std::fs::File;
use std::io::{BufReader, Read};

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
