use std::fmt::Display;
use std::string::FromUtf8Error;

use derive_more::{DebugCustom, From};
use tokio::task::JoinError;

#[derive(From, DebugCustom)]
pub enum Error {
    #[debug(fmt = "Std IO Error")]
    IO(std::io::Error),

    #[debug(fmt = "Join Error")]
    Join(JoinError),

    #[debug(fmt = "From Utf8 Error")]
    FromUtf8(FromUtf8Error),

    #[debug(fmt = "Prettier formatting failed: {}", _0)]
    PrettierFormattingFailed(String),

    #[debug(fmt = "No file extension found")]
    FileExtensionNotFound,

    #[debug(fmt = "Unsupported file type")]
    UnsupportedFiletype,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Result<A> = std::result::Result<A, Error>;
