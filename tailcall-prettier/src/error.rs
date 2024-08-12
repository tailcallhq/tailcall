use std::borrow::Cow;
use std::string::FromUtf8Error;
use std::sync::Arc;

use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error)]
pub enum Error {
    #[error("Std IO Error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Join Error: {0}")]
    Join(#[from] JoinError),

    #[error("From Utf8 Error: {0}")]
    FromUtf8(#[from] FromUtf8Error),

    #[error("Prettier formatting failed: {0}")]
    PrettierFormattingFailed(String),

    #[error("{0} command was not found. Ensure you have it installed and available in the PATH")]
    CommandNotFound(String),

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

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl Error {
    pub fn with_context(self, context: String) -> Self {
        Error::Context { source: Arc::new(self), context }
    }

    pub fn from_io_error(command: Cow<'static, str>) -> impl Fn(std::io::Error) -> Self {
        move |error| match error.kind() {
            std::io::ErrorKind::NotFound => Error::CommandNotFound(command.to_string()),
            _ => Error::IO(error),
        }
    }
}

pub type Result<A> = std::result::Result<A, Error>;
