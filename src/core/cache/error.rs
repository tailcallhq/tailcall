use std::fmt::Display;
use std::sync::Arc;

use derive_more::{DebugCustom, From};

#[derive(From, DebugCustom, Clone)]
pub enum Error {
    #[debug(fmt = "Serde Json Error: {}", _0)]
    SerdeJson(Arc<serde_json::Error>),

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

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SerdeJson(error) => write!(f, "Serde Json Error: {}", error),
            Error::Kv(error) => write!(f, "Kv Error: {}", error),
        }
    }
}
