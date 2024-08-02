use std::{string::FromUtf8Error, sync::Arc};

use derive_more::From;

#[derive(From, thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("HTTP request failed with status code: {status_code}")]
    RequestFailed { status_code: u16 },

    #[error("Timeout occurred while making the HTTP request")]
    Timeout,

    #[error("Failed to parse the response body")]
    ResponseParse,

    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },

    #[error("Reqwest Middleware Error: {}", _0)]
    ReqwestMiddleware(Arc<reqwest_middleware::Error>),

    #[error("Tonic Status Error: {}", _0)]
    TonicStatus(tonic::Status),

    #[error("Reqwest Error: {}", _0)]
    Reqwest(Arc<reqwest::Error>),

    #[error("Serde Json Error: {}", _0)]
    SerdeJson(Arc<serde_json::Error>),

    #[error("Unable to find key {0} in query params")]
    #[from(ignore)]
    KeyNotFound(String),

    #[error("Invalid Status Code: {}", _0)]
    InvalidStatusCode(Arc<hyper::http::status::InvalidStatusCode>),

    #[error("Status Code error")]
    StatusCode,

    #[error("Invalid Header Value: {}", _0)]
    InvalidHeaderValue(Arc<hyper::header::InvalidHeaderValue>),

    #[error("Invalid Header Name: {}", _0)]
    InvalidHeaderName(Arc<hyper::header::InvalidHeaderName>),

    #[error("No mock found for request: {method} {url} in {spec_path}")]
    NoMockFound {
        method: String,
        url: String,
        spec_path: String,
    },

    #[error("Hyper Error: {}", _0)]
    Hyper(Arc<hyper::Error>),

    #[error("Utf8 Error: {}", _0)]
    Utf8(FromUtf8Error),

    #[error("Invalid request host")]
    InvalidRequestHost,

    #[error("Hyper Http Error: {}", _0)]
    HyperHttp(Arc<hyper::http::Error>),
}

impl From<reqwest_middleware::Error> for Error {
    fn from(error: reqwest_middleware::Error) -> Self {
        Error::ReqwestMiddleware(Arc::new(error))
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::Reqwest(Arc::new(error))
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::SerdeJson(Arc::new(error))
    }
}

impl From<hyper::http::status::InvalidStatusCode> for Error {
    fn from(error: hyper::http::status::InvalidStatusCode) -> Self {
        Error::InvalidStatusCode(Arc::new(error))
    }
}

impl From<hyper::header::InvalidHeaderValue> for Error {
    fn from(error: hyper::header::InvalidHeaderValue) -> Self {
        Error::InvalidHeaderValue(Arc::new(error))
    }
}

impl From<hyper::header::InvalidHeaderName> for Error {
    fn from(error: hyper::header::InvalidHeaderName) -> Self {
        Error::InvalidHeaderName(Arc::new(error))
    }
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Self {
        Error::Hyper(Arc::new(error))
    }
}

impl From<hyper::http::Error> for Error {
    fn from(error: hyper::http::Error) -> Self {
        Error::HyperHttp(Arc::new(error))
    }
}

pub type Result<A> = std::result::Result<A, Error>;

