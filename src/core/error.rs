use std::fmt::Display;

use derive_more::From;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Worker Error")]
    Worker(worker::Error),
}

pub mod worker {
    use std::sync::Arc;

    use derive_more::{DebugCustom, From};
    use tokio::task::JoinError;

    #[derive(From, DebugCustom, Clone)]
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
        HyperHeaderStr(Arc<hyper::header::ToStrError>),

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
        Anyhow(Arc<anyhow::Error>),
    }

    impl From<serde_json::Error> for Error {
        fn from(error: serde_json::Error) -> Self {
            Error::SerdeJson(Arc::new(error))
        }
    }

    impl From<hyper::header::ToStrError> for Error {
        fn from(error: hyper::header::ToStrError) -> Self {
            Error::HyperHeaderStr(Arc::new(error))
        }
    }

    impl From<JoinError> for Error {
        fn from(error: JoinError) -> Self {
            Error::Join(Arc::new(error))
        }
    }

    impl From<anyhow::Error> for Error {
        fn from(error: anyhow::Error) -> Self {
            Error::Anyhow(Arc::new(error))
        }
    }

    pub type Result<A> = std::result::Result<A, Error>;
}

impl Display for worker::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            worker::Error::InitializationFailed => write!(f, "Failed to initialize worker"),
            worker::Error::Communication => write!(f, "Worker communication error"),
            worker::Error::SerdeJson(error) => write!(f, "Serde Json Error: {}", error),
            worker::Error::RequestCloneFailed => write!(f, "Request Clone Failed"),
            worker::Error::HyperHeaderStr(error) => {
                write!(f, "Hyper Header To Str Error: {}", error)
            }
            worker::Error::JsRuntimeStopped => write!(f, "JS Runtime Stopped Error"),
            worker::Error::CLI(msg) => write!(f, "CLI Error: {}", msg),
            worker::Error::Join(error) => write!(f, "Join Error: {}", error),
            worker::Error::RuntimeNotInitialized => write!(f, "Runtime not initialized"),
            worker::Error::InvalidFunction(function_name) => {
                write!(f, "{} is not a function", function_name)
            }
            worker::Error::Rquickjs(error) => write!(f, "Rquickjs error: {}", error),
            worker::Error::DeserializeFailed(error) => write!(f, "Deserialize Failed: {}", error),
            worker::Error::GlobalThisNotInitialised(error) => write!(f, "globalThis not initialized: {}", error),
            worker::Error::FunctionValueParseError(error, name) => write!(f, "Error: {}\nUnable to parse value from js function: {} maybe because it's not returning a string?", error, name),
            worker::Error::Anyhow(msg) => write!(f, "Error: {}", msg),
        }
    }
}

pub mod cache {
    use std::sync::Arc;

    use derive_more::{DebugCustom, From};

    use super::worker;

    #[derive(From, DebugCustom, Clone)]
    pub enum Error {
        #[debug(fmt = "Serde Json Error: {}", _0)]
        SerdeJson(Arc<serde_json::Error>),

        #[debug(fmt = "Worker Error: {}", _0)]
        Worker(worker::Error),

        #[debug(fmt = "Kv Error: {}", _0)]
        #[from(ignore)]
        Kv(String),
    }

    impl From<serde_json::Error> for Error {
        fn from(error: serde_json::Error) -> Self {
            Error::SerdeJson(Arc::new(error))
        }
    }

    pub type Result<A> = std::result::Result<A, Error>;
}

impl Display for cache::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            cache::Error::SerdeJson(error) => write!(f, "Serde Json Error: {}", error),
            cache::Error::Worker(error) => write!(f, "Worker Error: {}", error),
            cache::Error::Kv(error) => write!(f, "Kv Error: {}", error),
        }
    }
}

pub type Result<A, E> = std::result::Result<A, E>;
