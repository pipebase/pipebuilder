use std::fmt::Debug;
use std::{env, io, net, result};
use thiserror::Error;

#[derive(Debug)]
pub struct Error(Box<ErrorImpl>);

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum ErrorImpl {
    #[error("addr parse exception")]
    AddrParse(#[from] net::AddrParseError),
    #[error("env exception")]
    Env(#[from] env::VarError),
    #[error("io exception")]
    Io(#[from] io::Error),
    #[error("tonic transport exception")]
    TonicTransport(#[from] tonic::transport::Error),
    #[error("yaml exception")]
    Yaml(#[from] serde_yaml::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error(Box::new(ErrorImpl::Io(err)))
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error(Box::new(ErrorImpl::Yaml(err)))
    }
}

impl From<env::VarError> for Error {
    fn from(err: env::VarError) -> Self {
        Error(Box::new(ErrorImpl::Env(err)))
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(err: tonic::transport::Error) -> Self {
        Error(Box::new(ErrorImpl::TonicTransport(err)))
    }
}

impl From<net::AddrParseError> for Error {
    fn from(err: net::AddrParseError) -> Self {
        Error(Box::new(ErrorImpl::AddrParse(err)))
    }
}
