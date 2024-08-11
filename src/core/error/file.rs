use std::fmt::Display;
use std::string::FromUtf8Error;

use derive_more::{DebugCustom, From};

#[derive(From, DebugCustom)]
pub enum Error {
    #[debug(fmt = "No such file or directory (os error 2)")]
    NotFound,

    #[debug(fmt = "No permission to access the file")]
    NoPermission,

    #[debug(fmt = "Access denied")]
    AccessDenied,

    #[debug(fmt = "Invalid file format")]
    InvalidFormat,

    #[debug(fmt = "Invalid file path")]
    InvalidFilePath,

    #[debug(fmt = "Invalid OS string")]
    InvalidOsString,

    #[debug(fmt = "Failed to read file : {}: {}", path, error)]
    FileReadFailed { path: String, error: String },

    #[debug(fmt = "Failed to write file : {}: {}", path, error)]
    #[from(ignore)]
    FileWriteFailed { path: String, error: String },

    #[debug(fmt = "Std IO Error: {}", _0)]
    StdIO(std::io::Error),

    #[debug(fmt = "Utf8 Error: {}", _0)]
    Utf8(FromUtf8Error),

    #[debug(fmt = "File writing not supported on Lambda.")]
    LambdaFileWriteNotSupported,

    #[debug(fmt = "Cannot write to a file in an execution spec")]
    ExecutionSpecFileWriteFailed,

    #[debug(fmt = "Cloudflare Worker Execution Error : {}", _0)]
    #[from(ignore)]
    Cloudflare(String),

    #[debug(fmt = "File IO is not supported")]
    FileIONotSupported,

    #[debug(fmt = "Error : {}", _0)]
    Anyhow(anyhow::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotFound => write!(f, "No such file or directory (os error 2)"),
            Error::NoPermission => write!(f, "No permission to access the file"),
            Error::AccessDenied => write!(f, "Access denied"),
            Error::InvalidFormat => write!(f, "Invalid file format"),
            Error::InvalidFilePath => write!(f, "Invalid file path"),
            Error::InvalidOsString => write!(f, "Invalid OS string"),
            Error::FileReadFailed { path, error } => {
                write!(f, "Failed to read file: {}: {}", path, error)
            }
            Error::FileWriteFailed { path, error } => {
                write!(f, "Failed to write file: {}: {}", path, error)
            }
            Error::StdIO(error) => write!(f, "Std IO Error: {}", error),
            Error::Utf8(error) => write!(f, "Utf8 Error: {}", error),
            Error::LambdaFileWriteNotSupported => {
                write!(f, "File writing not supported on Lambda.")
            }
            Error::ExecutionSpecFileWriteFailed => {
                write!(f, "Cannot write to a file in an execution spec")
            }
            Error::Cloudflare(error) => {
                write!(f, "Cloudflare Worker Execution Error: {}", error)
            }
            Error::FileIONotSupported => {
                write!(f, "File IO is not supported")
            }
            Error::Anyhow(msg) => write!(f, "Error: {}", msg),
        }
    }
}
