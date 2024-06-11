use std::string::FromUtf8Error;

use derive_more::From;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Std IO Error")]
    StdIO(std::io::Error),

    #[error("Utf8 Error")]
    Utf8(FromUtf8Error),

    #[error("File Error")]
    File(file::Error),

    #[error("Http Error")]
    Http(http::Error),

    #[error("Worker Error")]
    Worker(worker::Error),
}

pub mod file {
    use std::string::FromUtf8Error;

    use derive_more::From;

    #[derive(From, thiserror::Error, Debug)]
    #[error("Error occurred with file '{file_path}': {error}")]
    pub struct Error{pub file_path: String, pub error: FileError}

    #[derive(From, thiserror::Error, Debug)]
    pub enum FileError {
        #[error("File not found")]
        NotFound,

        #[error("No permission to access the file")]
        NoPermission,
        
        #[error("Access denied")]
        AccessDenied,
        
        #[error("Invalid file format")]
        InvalidFormat,

        #[error("Failed to read file : {0}")]
        FileReadFailed(String),

        #[error("Failed to write file : {0}")]
        #[from(ignore)]
        FileWriteFailed(String),

        #[error("Std IO Error")]
        StdIO(std::io::Error),

        #[error("Utf8 Error")]
        Utf8(FromUtf8Error),
    }
}

pub mod http {
    use derive_more::From;

    #[derive(From, thiserror::Error, Debug)]
    pub enum HttpError {
        #[error("HTTP request failed with status code: {status_code}")]
        RequestFailed { status_code: u16 },

        #[error("Timeout occurred while making the HTTP request")]
        Timeout,

        #[error("Failed to parse the response body")]
        ResponseParseError,

        #[error("Invalid URL: {url}")]
        InvalidUrl { url: String },

        #[error("Reqwest Middleware Error")]
        ReqwestMiddleware(reqwest_middleware::Error),

        #[error("Tonic Status Error")]
        TonicStatus(tonic::Status),
    }

    #[derive(From, thiserror::Error, Debug)]
    #[error("HTTP Error: {source}")]
    pub struct Error {
        pub source: HttpError,
    }
}

pub mod worker {
    use derive_more::From;

    #[derive(From, thiserror::Error, Debug)]
    pub enum WorkerError {
        #[error("Failed to initialize worker")]
        InitializationFailed,

        #[error("Worker execution error")]
        ExecutionFailed,

        #[error("Worker communication error")]
        CommunicationError,

        #[error("Serde Json Error")]
        SerdeJson(serde_json::Error),

        #[error("Request Clone Failed")]
        RequestCloneFailed,

        #[error("Hyper Header To Str Error")]
        HyperHeaderStr(hyper::header::ToStrError),

        #[error("CLI Error")]
        CLIError(crate::cli::error::Error),

        #[error("JS Runtime Stopped Error")]
        JsRuntimeStopped,
    }

    #[derive(From, thiserror::Error, Debug)]
    #[error("Worker Error: {source}")]
    pub struct Error {
        pub source: WorkerError,
    }
}

pub type Result<A, E> = std::result::Result<A, E>;
