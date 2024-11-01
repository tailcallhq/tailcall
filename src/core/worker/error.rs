use std::fmt::Display;
use std::sync::Arc;

use derive_more::{DebugCustom, From};
use miette::Diagnostic;
use tokio::task::JoinError;

#[derive(From, DebugCustom, Clone, Diagnostic, thiserror::Error)]
pub enum Error {
    #[debug(fmt = "Failed to initialize worker")]
    InitializationFailed,

    #[debug(fmt = "Worker communication error")]
    Communication,

    #[debug(fmt = "Serde Json Error: {}", _0)]
    SerdeJson(Arc<serde_json::Error>),

    #[debug(fmt = "Request Clone Failed")]
    RequestCloneFailed,

    #[debug(fmt = "Hyper Header To Str Error: {}", _0)]
    HyperHeaderStr(Arc<http::header::ToStrError>),

    #[debug(fmt = "JS Runtime Stopped Error")]
    JsRuntimeStopped,

    #[debug(fmt = "CLI Error : {}", _0)]
    CLI(String),

    #[debug(fmt = "Join Error : {}", _0)]
    Join(Arc<JoinError>),

    #[debug(fmt = "Runtime not initialized")]
    RuntimeNotInitialized,

    #[debug(fmt = "{} is not a function", _0)]
    #[from(ignore)]
    InvalidFunction(String),

    #[debug(fmt = "Rquickjs Error: {}", _0)]
    #[from(ignore)]
    Rquickjs(String),

    #[debug(fmt = "Deserialize Failed: {}", _0)]
    #[from(ignore)]
    DeserializeFailed(String),

    #[debug(fmt = "globalThis not initialized: {}", _0)]
    #[from(ignore)]
    GlobalThisNotInitialised(String),

    #[debug(
        fmt = "Error: {}\nUnable to parse value from js function: {} maybe because it's not returning a string?",
        _0,
        _1
    )]
    FunctionValueParseError(String, String),

    #[debug(fmt = "Error : {}", _0)]
    Miette(Arc<miette::Error>),
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::SerdeJson(Arc::new(error))
    }
}

impl From<http::header::ToStrError> for Error {
    fn from(error: http::header::ToStrError) -> Self {
        Error::HyperHeaderStr(Arc::new(error))
    }
}

impl From<JoinError> for Error {
    fn from(error: JoinError) -> Self {
        Error::Join(Arc::new(error))
    }
}

impl From<miette::Error> for Error {
    fn from(error: miette::Error) -> Self {
        Error::Miette(Arc::new(error))
    }
}

pub type Result<A> = std::result::Result<A, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InitializationFailed => write!(f, "Failed to initialize worker"),
            Error::Communication => write!(f, "Worker communication error"),
            Error::SerdeJson(error) => write!(f, "Serde Json Error: {}", error),
            Error::RequestCloneFailed => write!(f, "Request Clone Failed"),
            Error::HyperHeaderStr(error) => {
                write!(f, "Hyper Header To Str Error: {}", error)
            }
            Error::JsRuntimeStopped => write!(f, "JS Runtime Stopped Error"),
            Error::CLI(msg) => write!(f, "CLI Error: {}", msg),
            Error::Join(error) => write!(f, "Join Error: {}", error),
            Error::RuntimeNotInitialized => write!(f, "Runtime not initialized"),
            Error::InvalidFunction(function_name) => {
                write!(f, "{} is not a function", function_name)
            }
            Error::Rquickjs(error) => write!(f, "Rquickjs error: {}", error),
            Error::DeserializeFailed(error) => write!(f, "Deserialize Failed: {}", error),
            Error::GlobalThisNotInitialised(error) => write!(f, "globalThis not initialized: {}", error),
            Error::FunctionValueParseError(error, name) => write!(f, "Error: {}\nUnable to parse value from js function: {} maybe because it's not returning a string?", error, name),
            Error::Miette(msg) => write!(f, "Error: {}", msg),
        }
    }
}
