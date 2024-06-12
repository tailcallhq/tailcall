use std::string::FromUtf8Error;

use derive_more::From;
use inquire::InquireError;
use prost_reflect::DescriptorError;

use super::config::UnsupportedConfigFormat;
use super::grpc::error::Error as GrpcError;
use super::rest::error::Error as RestError;
use super::valid::ValidationError;
use crate::cli::error::Error as CLIError;
use crate::core::errata::Errata as ErrataError;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Std IO Error")]
    StdIO(std::io::Error),

    #[error("Utf8 Error")]
    Utf8(FromUtf8Error),

    #[error("Validation Error : {0}")]
    ValidationError(ValidationError<std::string::String>),

    #[error("Serde Json Error")]
    SerdeJson(serde_json::Error),

    #[error("Serde Yaml Error")]
    SerdeYaml(serde_yaml::Error),

    #[error("Descriptor Error")]
    Descriptor(DescriptorError),

    #[error("Expected fully-qualified name for reference type but got {0}")]
    InvalidReferenceTypeName(String),

    #[error("Package name is required")]
    PackageNameNotFound,

    #[error("Protox Parse Error")]
    ProtoxParse(protox_parse::ParseError),

    #[error("URL Parse Error")]
    UrlParse(url::ParseError),

    #[error("Unable to extract content of google well-known proto file")]
    GoogleProtoFileContentNotExtracted,

    #[error("Unsupported Config Format")]
    UnsupportedConfigFormat(UnsupportedConfigFormat),

    #[error("Couldn't find definitions for service ServerReflection")]
    MissingServerReflectionDefinitions,

    #[error("Grpc Error")]
    GrpcError(GrpcError),

    #[error("Serde Path To Error")]
    SerdePathToError(serde_path_to_error::Error<serde_json::Error>),

    #[error("Rest Error")]
    RestError(RestError),

    #[error("Expected fileDescriptorResponse but found none")]
    MissingFileDescriptorResponse,

    #[error("Prost Decode Error")]
    ProstDecodeError(prost::DecodeError),

    #[error("Received empty fileDescriptorProto")]
    EmptyFileDescriptorProto,

    #[error("Failed to decode fileDescriptorProto from BASE64")]
    FileDescriptorProtoDecodeFailed,

    #[error("File Error")]
    File(file::FileError),

    #[error("Http Error")]
    Http(http::HttpError),

    #[error("Worker Error")]
    Worker(worker::WorkerError),

    #[error("CLI Error")]
    CLIError(CLIError),

    #[error("Inquire Error")]
    InquireError(InquireError),

    #[error("Errata Error")]
    Errata(ErrataError),
}

pub mod file {
    use std::string::FromUtf8Error;

    use derive_more::From;

    #[derive(From, thiserror::Error, Debug)]
    #[error("Error occurred with file '{file_path}': {error}")]
    pub struct Error {
        pub file_path: String,
        pub error: FileError,
    }

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

        #[error("Reqwest Error")]
        ReqwestError(reqwest::Error),

        #[error("Serde Json Error")]
        SerdeJson(serde_json::Error),

        #[error("Unable to find key {0} in query params")]
        #[from(ignore)]
        KeyNotFound(String),
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

pub mod graphql {
    use derive_more::From;

    use super::http::HttpError;

    #[derive(From, thiserror::Error, Debug)]
    pub enum GraphqlError {
        #[error("Serde Json Error")]
        SerdeJson(serde_json::Error),

        #[error("HTTP Error")]
        Http(HttpError),
    }

    #[derive(From, thiserror::Error, Debug)]
    #[error("Graphql Error: {source}")]
    pub struct Error {
        pub source: GraphqlError,
    }
}

pub type Result<A, E> = std::result::Result<A, E>;
