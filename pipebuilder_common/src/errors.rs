use std::fmt::{Debug, Display};
use std::{env, io, net, result, string::FromUtf8Error};
use thiserror::Error;

#[derive(Debug)]
pub struct Error(Box<ErrorImpl>);

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum ErrorImpl {
    #[error("addr parse error, detail: {0:?}")]
    AddrParse(#[from] net::AddrParseError),
    #[error("env error, detail: {0:?}")]
    Env(#[from] env::VarError),
    #[error("etcd client error, detail: {0:?}")]
    Etcd(#[from] etcd_client::Error),
    #[error("io error, detail: {0:?}")]
    Io(#[from] io::Error),
    #[error("json error, detail: {0:?}")]
    Json(#[from] serde_json::Error),
    #[error("tonic transport error, detail: {0:?}")]
    TonicTransport(#[from] tonic::transport::Error),
    #[error("yaml error, detail: {0:?}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("rpc error, detail: {0:?}")]
    Rpc(#[from] tonic::Status),
    #[error("pipegen error, detail: {0:?}")]
    Pipegen(#[from] pipegen::error::Error),
    #[error("toml deserialize error, detail: {0:?}")]
    TomlDe(#[from] toml::de::Error),
    #[error("toml Serialize error, detail: {0:?}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("utf8 error, detail: {0:?}")]
    Utf8(#[from] FromUtf8Error),
    #[error("cargo {cmd:?} error, code: {code:?}, message: {msg:?}")]
    Cargo { cmd: String, code: i32, msg: String },
    #[error("reqwest error, detail: {0:?}")]
    Reqwest(#[from] reqwest::Error),
    #[error("http header name error, detail: {0:?}")]
    HttpHeaderName(#[from] http::header::InvalidHeaderName),
    #[error("http header value error, detail: {0:?}")]
    HttpHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error(
        "api client error, status_code: {status_code:?}, reason: {reason:?}, message: {message:?}"
    )]
    ApiClient {
        status_code: u16,
        reason: String,
        message: String,
    },
    #[error(
        "api server error, status_code: {status_code:?}, reason: {reason:?}, message: {message:?}"
    )]
    ApiServer {
        status_code: u16,
        reason: String,
        message: String,
    },
    #[error("invalid api request, message: {message:?}")]
    ApiRequest { message: String },
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

pub fn api_client_error(
    status_code: u16,
    reason: Option<String>,
    message: Option<String>,
) -> Error {
    Error(Box::new(ErrorImpl::ApiClient {
        status_code,
        reason: reason.unwrap_or_default(),
        message: message.unwrap_or_default(),
    }))
}

pub fn api_server_error(
    status_code: u16,
    reason: Option<String>,
    message: Option<String>,
) -> Error {
    Error(Box::new(ErrorImpl::ApiServer {
        status_code,
        reason: reason.unwrap_or_default(),
        message: message.unwrap_or_default(),
    }))
}

pub fn invalid_api_request(message: String) -> Error {
    Error(Box::new(ErrorImpl::ApiRequest { message }))
}

// rpc status
pub fn rpc_internal_error(error: Error) -> tonic::Status {
    tonic::Status::internal(format!("{:#?}", error))
}

pub fn rpc_not_found(message: &str) -> tonic::Status {
    tonic::Status::not_found(message)
}
