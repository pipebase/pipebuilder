use crate::Error;
use serde::de::DeserializeOwned;
use std::fs::File;

pub fn read_file<P>(path: P) -> Result<File, Error>
where
    P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(path)?;
    Ok(file)
}

pub fn parse_config<C>(file: File) -> Result<C, Error>
where
    C: DeserializeOwned,
{
    let config = serde_yaml::from_reader::<std::fs::File, C>(file)?;
    Ok(config)
}
