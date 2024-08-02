use std::fmt::Display;
use std::string::FromUtf8Error;
use std::sync::Arc;

use derive_more::From;

use super::worker;

use super::config::UnsupportedConfigFormat;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Worker Error: {}", _0)]
    Worker(worker::Error),

    #[error("File Error: {}", _0)]
    File(file::Error),

    #[error("Inquire Error: {}", _0)]
    Inquire(String),

    #[error("Serde Yaml Error: {}", _0)]
    SerdeYaml(serde_yaml::Error),

    #[error("Package name is required")]
    PackageNameNotFound,

    #[error("Protox Parse Error: {}", _0)]
    ProtoxParse(protox_parse::ParseError),

    #[error("Unable to extract content of google well-known proto file")]
    GoogleProtoFileContentNotExtracted,

    #[error("Utf8 Error: {}", _0)]
    Utf8(FromUtf8Error),

    #[error("Unsupported Config Format: {}", _0)]
    UnsupportedConfigFormat(UnsupportedConfigFormat),

    #[error("Unsupported File Format")]
    UnsupportedFileFormat,

    #[error("Serde Json Error: {}", _0)]
    SerdeJson(serde_json::Error),

    #[error("Unable to determine path")]
    PathDeterminationFailed,

    #[error("Schema mismatch Error")]
    SchemaMismatch,

    #[error("{}\n\nCaused by:\n    {}", context, source)]
    Context { source: Arc<Error>, context: String },

    #[error("Error: {}", _0)]
    Anyhow(anyhow::Error),
}

impl Error {
    pub fn with_context(self, context: String) -> Self {
        Error::Context { source: Arc::new(self), context }
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

    pub type Result<A> = std::result::Result<A, Error>;
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
            file::Error::StdIO(error) => write!(f, "Std IO Error: {}", error),
            file::Error::Utf8(error) => write!(f, "Utf8 Error: {}", error),
            file::Error::LambdaFileWriteNotSupported => {
                write!(f, "File writing not supported on Lambda.")
            }
            file::Error::ExecutionSpecFileWriteFailed => {
                write!(f, "Cannot write to a file in an execution spec")
            }
            file::Error::Cloudflare(error) => {
                write!(f, "Cloudflare Worker Execution Error: {}", error)
            }
            file::Error::FileIONotSupported => {
                write!(f, "File IO is not supported")
            }
            file::Error::Anyhow(msg) => write!(f, "Error: {}", msg),
        }
    }
}

pub type Result<A, E> = std::result::Result<A, E>;
