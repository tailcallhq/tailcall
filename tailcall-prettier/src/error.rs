use std::fmt::Display;
use std::string::FromUtf8Error;
use std::sync::Arc;

use derive_more::{DebugCustom, From};
use tokio::task::JoinError;

#[derive(From, DebugCustom)]
pub enum Error {
    #[debug(fmt = "Std IO Error: {}", _0)]
    IO(std::io::Error),

    #[debug(fmt = "Join Error: {}", _0)]
    Join(JoinError),

    #[debug(fmt = "From Utf8 Error: {}", _0)]
    FromUtf8(FromUtf8Error),

    #[debug(fmt = "Prettier formatting failed: {}", _0)]
    PrettierFormattingFailed(String),

    #[debug(fmt = "No file extension found")]
    FileExtensionNotFound,

    #[debug(fmt = "Unsupported file type")]
    UnsupportedFiletype,

    #[debug(fmt = "{}\n\nCaused by:\n    {}", context, source)]
    Context { source: Arc<Error>, context: String },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(error) => write!(f, "Std IO Error: {}", error),
            Error::Join(error) => write!(f, "Join Error: {}", error),
            Error::FromUtf8(error) => write!(f, "From Utf8 Error: {}", error),
            Error::PrettierFormattingFailed(msg) => {
                write!(f, "Prettier formatting failed: {}", msg)
            }
            Error::FileExtensionNotFound => write!(f, "No file extension found"),
            Error::UnsupportedFiletype => write!(f, "Unsupported file type"),
            Error::Context { source, context } => {
                write!(f, "{}\n\nCaused by:\n    {}", context, source)
            }
        }
    }
}

impl Error {
    pub fn with_context(self, context: String) -> Self {
        Error::Context { source: Arc::new(self), context }
    }
}

pub type Result<A> = std::result::Result<A, Error>;
