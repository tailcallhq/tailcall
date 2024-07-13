use std::fmt::Display;

use derive_more::From;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Worker Error")]
    Worker(worker::Error),
}

pub mod worker {
    use derive_more::{DebugCustom, From};
    use tokio::task::JoinError;

    #[derive(From, DebugCustom)]
    pub enum Error {
        #[debug(fmt = "Failed to initialize worker")]
        InitializationFailed,

        #[debug(fmt = "Worker communication error")]
        Communication,

        #[debug(fmt = "Serde Json Error")]
        SerdeJson(serde_json::Error),

        #[debug(fmt = "Request Clone Failed")]
        RequestCloneFailed,

        #[debug(fmt = "Hyper Header To Str Error")]
        HyperHeaderStr(hyper::header::ToStrError),

        #[debug(fmt = "JS Runtime Stopped Error")]
        JsRuntimeStopped,

        #[debug(fmt = "CLI Error : {}", _0)]
        CLI(String),

        #[debug(fmt = "Join Error : {}", _0)]
        Join(JoinError),

        #[debug(fmt = "Error : {}", _0)]
        Anyhow(anyhow::Error),
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
            worker::Error::HyperHeaderStr(error) => write!(f, "Hyper Header To Str Error: {}", error),
            worker::Error::JsRuntimeStopped => write!(f, "JS Runtime Stopped Error"),
            worker::Error::CLI(msg) => write!(f, "CLI Error: {}", msg),            
            worker::Error::Join(error) => write!(f, "Join Error: {}", error),
            worker::Error::Anyhow(msg) => write!(f, "Error: {}", msg),
        }
    }
}

pub type Result<A, E> = std::result::Result<A, E>;
