use std::string::FromUtf8Error;
use std::sync::Arc;

use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Std IO Error: {0}")]
    IO(#[from] std::io::Error),

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

impl Error {
    pub fn with_context(self, context: String) -> Self {
        Error::Context { source: Arc::new(self), context }
    }
}

pub type Result<A> = std::result::Result<A, Error>;
