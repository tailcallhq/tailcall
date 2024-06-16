use std::string::FromUtf8Error;

use derive_more::From;
use tokio::task::JoinError;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Std IO Error")]
    IO(std::io::Error),

    #[error("Join Error")]
    Join(JoinError),

    #[error("From Utf8 Error")]
    FromUtf8(FromUtf8Error),

    #[error("Prettier formatting failed: {0}")]
    PrettierFormattingFailed(String),

    #[error("No file extension found")]
    FileExtensionNotFound,

    #[error("Unsupported file type")]
    UnsupportedFiletype,
}

pub type Result<A> = std::result::Result<A, Error>;
