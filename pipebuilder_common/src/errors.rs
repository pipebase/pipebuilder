use std::fmt::Debug;
use std::{env, io, net, result, string::FromUtf8Error};
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
    #[error("etcd client exception")]
    Etcd(#[from] etcd_client::Error),
    #[error("io exception")]
    Io(#[from] io::Error),
    #[error("json exception")]
    Json(#[from] serde_json::Error),
    #[error("tonic transport exception")]
    TonicTransport(#[from] tonic::transport::Error),
    #[error("yaml exception")]
    Yaml(#[from] serde_yaml::Error),
    #[error("rpc exception")]
    Rpc(#[from] tonic::Status),
    #[error("pipegen exception")]
    Pipegen(#[from] pipegen::error::Error),
    #[error("toml deserialize exception")]
    TomlDe(#[from] toml::de::Error),
    #[error("toml Serialize exception")]
    TomlSer(#[from] toml::ser::Error),
    #[error("utf8 exception")]
    Utf8(#[from] FromUtf8Error),
    #[error("cargo {cmd:?} error")]
    Cargo { cmd: String, code: i32, msg: String },
    #[error("reqwest exception")]
    Reqwest(#[from] reqwest::Error),
    #[error("http header name exception")]
    HttpHeaderName(#[from] http::header::InvalidHeaderName),
    #[error("http header value exception")]
    HttpHeaderValue(#[from] http::header::InvalidHeaderValue),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error(Box::new(ErrorImpl::Io(err)))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error(Box::new(ErrorImpl::Json(err)))
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

impl From<etcd_client::Error> for Error {
    fn from(err: etcd_client::Error) -> Self {
        Error(Box::new(ErrorImpl::Etcd(err)))
    }
}

impl From<tonic::Status> for Error {
    fn from(status: tonic::Status) -> Self {
        Error(Box::new(ErrorImpl::Rpc(status)))
    }
}

impl From<pipegen::error::Error> for Error {
    fn from(err: pipegen::error::Error) -> Self {
        Error(Box::new(ErrorImpl::Pipegen(err)))
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error(Box::new(ErrorImpl::TomlDe(err)))
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error(Box::new(ErrorImpl::TomlSer(err)))
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Error(Box::new(ErrorImpl::Utf8(err)))
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error(Box::new(ErrorImpl::Reqwest(err)))
    }
}

impl From<http::header::InvalidHeaderName> for Error {
    fn from(err: http::header::InvalidHeaderName) -> Self {
        Error(Box::new(ErrorImpl::HttpHeaderName(err)))
    }
}

impl From<http::header::InvalidHeaderValue> for Error {
    fn from(err: http::header::InvalidHeaderValue) -> Self {
        Error(Box::new(ErrorImpl::HttpHeaderValue(err)))
    }
}

pub fn cargo_error(cmd: &str, code: i32, msg: String) -> Error {
    Error(Box::new(ErrorImpl::Cargo {
        cmd: String::from(cmd),
        code,
        msg,
    }))
}
