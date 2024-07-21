use std::fmt::Display;
use std::string::FromUtf8Error;

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
    Anyhow(anyhow::Error),
}

pub mod worker {
    use std::sync::Arc;

    use derive_more::{DebugCustom, From};
    use tokio::task::JoinError;

    #[derive(From, DebugCustom, Clone)]
    pub enum Error {
        #[debug(fmt = "Failed to initialize worker")]
        InitializationFailed,

        #[debug(fmt = "Worker communication error")]
        Communication,

        #[debug(fmt = "Serde Json Error: {}", _0)]
        SerdeJson(Arc<serde_json::Error>),

        #[debug(fmt = "Request Clone Failed")]
        RequestCloneFailed,

        #[debug(fmt = "Hyper Header To Str Error: {}", _0)]
        HyperHeaderStr(Arc<hyper::header::ToStrError>),

        #[debug(fmt = "JS Runtime Stopped Error")]
        JsRuntimeStopped,

        #[debug(fmt = "CLI Error : {}", _0)]
        CLI(String),

        #[debug(fmt = "Join Error : {}", _0)]
        Join(Arc<JoinError>),

        #[debug(fmt = "Runtime not initialized")]
        RuntimeNotInitialized,

        #[debug(fmt = "{} is not a function", _0)]
        #[from(ignore)]
        InvalidFunction(String),

        #[debug(fmt = "Rquickjs Error: {}", _0)]
        #[from(ignore)]
        Rquickjs(String),

        #[debug(fmt = "Deserialize Failed: {}", _0)]
        #[from(ignore)]
        DeserializeFailed(String),

        #[debug(fmt = "globalThis not initialized: {}", _0)]
        #[from(ignore)]
        GlobalThisNotInitialised(String),

        #[debug(
            fmt = "Error: {}\nUnable to parse value from js function: {} maybe because it's not returning a string?",
            _0,
            _1
        )]
        FunctionValueParseError(String, String),

        #[debug(fmt = "Error : {}", _0)]
        Anyhow(Arc<anyhow::Error>),
    }

    impl From<serde_json::Error> for Error {
        fn from(error: serde_json::Error) -> Self {
            Error::SerdeJson(Arc::new(error))
        }
    }

    impl From<hyper::header::ToStrError> for Error {
        fn from(error: hyper::header::ToStrError) -> Self {
            Error::HyperHeaderStr(Arc::new(error))
        }
    }

    impl From<JoinError> for Error {
        fn from(error: JoinError) -> Self {
            Error::Join(Arc::new(error))
        }
    }

    impl From<anyhow::Error> for Error {
        fn from(error: anyhow::Error) -> Self {
            Error::Anyhow(Arc::new(error))
        }
    }

    pub type Result<A> = std::result::Result<A, Error>;
}

impl Display for worker::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            worker::Error::InitializationFailed => write!(f, "Failed to initialize worker"),
            worker::Error::Communication => write!(f, "Worker communication error"),
            worker::Error::SerdeJson(error) => write!(f, "Serde Json Error: {}", error),
            worker::Error::RequestCloneFailed => write!(f, "Request Clone Failed"),
            worker::Error::HyperHeaderStr(error) => {
                write!(f, "Hyper Header To Str Error: {}", error)
            }
            worker::Error::JsRuntimeStopped => write!(f, "JS Runtime Stopped Error"),
            worker::Error::CLI(msg) => write!(f, "CLI Error: {}", msg),
            worker::Error::Join(error) => write!(f, "Join Error: {}", error),
            worker::Error::RuntimeNotInitialized => write!(f, "Runtime not initialized"),
            worker::Error::InvalidFunction(function_name) => {
                write!(f, "{} is not a function", function_name)
            }
            worker::Error::Rquickjs(error) => write!(f, "Rquickjs error: {}", error),
            worker::Error::DeserializeFailed(error) => write!(f, "Deserialize Failed: {}", error),
            worker::Error::GlobalThisNotInitialised(error) => write!(f, "globalThis not initialized: {}", error),
            worker::Error::FunctionValueParseError(error, name) => write!(f, "Error: {}\nUnable to parse value from js function: {} maybe because it's not returning a string?", error, name),
            worker::Error::Anyhow(msg) => write!(f, "Error: {}", msg),
        }
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
            file::Error::Anyhow(msg) => write!(f, "Error: {}", msg),
        }
    }
}

pub type Result<A, E> = std::result::Result<A, E>;
