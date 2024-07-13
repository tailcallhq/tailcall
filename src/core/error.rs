use std::{fmt::Display, string::FromUtf8Error};

use derive_more::From;
use inquire::InquireError;

use super::config::UnsupportedConfigFormat;


#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Worker Error: {}", _0)]
    Worker(worker::Error),

    #[error("File Error: {}", _0)]
    File(file::Error),

    #[error("Inquire Error: {}", _0)]
    Inquire(InquireError),

    #[error("Serde Yaml Error: {}", _0)]
    SerdeYaml(serde_yaml::Error),

    #[error("Package name is required")]
    PackageNameNotFound,

    #[error("Protox Parse Error")]
    ProtoxParse(protox_parse::ParseError),

    #[error("Unable to extract content of google well-known proto file")]
    GoogleProtoFileContentNotExtracted,

    #[error("Utf8 Error: {}", _0)]
    Utf8(FromUtf8Error),

    #[error("Unsupported Config Format")]
    UnsupportedConfigFormat(UnsupportedConfigFormat),

    #[error("Unsupported File Format")]
    UnsupportedFileFormat,

    #[error("Serde Json Error: {}", _0)]
    SerdeJson(serde_json::Error),

    #[error("Unable to determine path")]
    PathDeterminationFailed,

    #[error("Schema mismatch Error")]
    SchemaMismatch,

    #[error("Error: {}", _0)]
    Anyhow(anyhow::Error)
}

pub mod worker {
    use derive_more::{DebugCustom, From};

    #[derive(From, DebugCustom)]
    pub enum Error {
        #[debug(fmt = "Failed to initialize worker")]
        InitializationFailed,

        #[debug(fmt = "Worker execution error")]
        ExecutionFailed,

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

        #[debug(fmt = "Error : {}", _0)]
        Anyhow(anyhow::Error),
    }
}

pub mod file {
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

        #[debug(fmt = "Failed to read file : {}", _0)]
        FileReadFailed(String),

        #[debug(fmt = "Failed to write file : {}", _0)]
        #[from(ignore)]
        FileWriteFailed(String),

        #[debug(fmt = "Std IO Error")]
        StdIO(std::io::Error),

        #[debug(fmt = "Utf8 Error")]
        Utf8(FromUtf8Error),

        #[debug(fmt = "File writing not supported on Lambda.")]
        LambdaFileWriteNotSupported,

        #[debug(fmt = "Cannot write to a file in an execution spec")]
        ExecutionSpecFileWriteFailed,

        #[debug(fmt = "Cloudflare Worker Execution Error : {}", _0)]
        #[from(ignore)]
        Cloudflare(String),

        #[debug(fmt = "Error : {}", _0)]
        Anyhow(anyhow::Error),
    }
}

impl Display for worker::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            worker::Error::InitializationFailed => write!(f, "Failed to initialize worker"),
            worker::Error::ExecutionFailed => write!(f, "Worker execution error"),
            worker::Error::Communication => write!(f, "Worker communication error"),
            worker::Error::SerdeJson(_) => write!(f, "Serde Json Error"),
            worker::Error::RequestCloneFailed => write!(f, "Request Clone Failed"),
            worker::Error::HyperHeaderStr(_) => write!(f, "Hyper Header To Str Error"),
            worker::Error::JsRuntimeStopped => write!(f, "JS Runtime Stopped Error"),
            worker::Error::CLI(msg) => write!(f, "CLI Error: {}", msg),
            worker::Error::Anyhow(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl Display for file::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            file::Error::NotFound => write!(f, "No such file or directory (os error 2)"),
            file::Error::NoPermission => write!(f, "No permission to access the file"),
            file::Error::AccessDenied => write!(f, "Access denied"),
            file::Error::InvalidFormat => write!(f, "Invalid file format"),
            file::Error::InvalidFilePath => write!(f, "Invalid file path"),
            file::Error::InvalidOsString => write!(f, "Invalid OS string"),
            file::Error::FileReadFailed(path) => write!(f, "Failed to read file: {}", path),
            file::Error::FileWriteFailed(path) => write!(f, "Failed to write file: {}", path),
            file::Error::StdIO(_) => write!(f, "Std IO Error"),
            file::Error::Utf8(_) => write!(f, "Utf8 Error"),
            file::Error::LambdaFileWriteNotSupported => {
                write!(f, "File writing not supported on Lambda.")
            }
            file::Error::ExecutionSpecFileWriteFailed => {
                write!(f, "Cannot write to a file in an execution spec")
            }
            file::Error::Cloudflare(error) => {
                write!(f, "Cloudflare Worker Execution Error: {}", error)
            }
            file::Error::Anyhow(msg) => write!(f, "Error: {}", msg),
        }
    }
}

pub type Result<A, E> = std::result::Result<A, E>;
