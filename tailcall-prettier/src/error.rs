use std::string::FromUtf8Error;
use std::sync::Arc;

use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error)]
pub enum Error {
    #[error("Std IO Error: {0}")]
    IO(#[source] std::io::Error),

    #[error("Join Error: {0}")]
    Join(#[from] JoinError),

    #[error("From Utf8 Error: {0}")]
    FromUtf8(#[from] FromUtf8Error),

    #[error("Prettier formatting failed: {0}")]
    PrettierFormattingFailed(String),

    #[error("Prettier command not found. Do you have it installed and available in the PATH?")]
    PrettierNotFound,

    #[error("No file extension found")]
    FileExtensionNotFound,

    #[error("Unsupported file type")]
    UnsupportedFiletype,

    #[error("{}\n\nCaused by:\n    {}", context, source)]
    Context {
        #[source]
        source: Arc<Error>,
        context: String,
    },
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => Error::PrettierNotFound,
            _ => Error::IO(error),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl Error {
    pub fn with_context(self, context: String) -> Self {
        Error::Context { source: Arc::new(self), context }
    }
}

pub type Result<A> = std::result::Result<A, Error>;
