use std::str::Utf8Error;
use std::string::FromUtf8Error;

use derive_more::From;
use inquire::InquireError;
use prost_reflect::DescriptorError;

use super::config::UnsupportedConfigFormat;
use super::grpc::error::Error as GrpcError;
use super::ir;
use super::rest::error::Error as RestError;
use super::valid::ValidationError;
use crate::cli::error::Error as CLIError;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Std IO Error")]
    StdIO(std::io::Error),

    #[error("Utf8 Error")]
    FromUtf8(FromUtf8Error),

    #[error("Validation Error : {0}")]
    Validation(ValidationError<std::string::String>),

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
    Grpc(GrpcError),

    #[error("Serde Path To Error")]
    SerdePath(serde_path_to_error::Error<serde_json::Error>),

    #[error("Rest Error")]
    Rest(RestError),

    #[error("Expected fileDescriptorResponse but found none")]
    MissingFileDescriptorResponse,

    #[error("Prost Decode Error")]
    ProstDecode(prost::DecodeError),

    #[error("Received empty fileDescriptorProto")]
    EmptyFileDescriptorProto,

    #[error("Failed to decode fileDescriptorProto from BASE64")]
    FileDescriptorProtoDecodeFailed,

    #[error("Invalid header value")]
    HyperInvalidHeaderValue(hyper::header::InvalidHeaderValue),

    #[error("Invalid header name")]
    HyperInvalidHeaderName(hyper::header::InvalidHeaderName),

    #[error("Hyper HTTP Error")]
    HyperHttpError(hyper::http::Error),

    #[error("Prometheus Error")]
    Prometheus(prometheus::Error),

    #[error("Hyper Error")]
    Hyper(hyper::Error),

    #[error("Invalid URI")]
    InvalidUri(hyper::http::uri::InvalidUri),

    #[error("Mustache cannot be converted to const value")]
    InvalidMustacheConstConversion,

    #[error("Protox Error")]
    Protox(protox::Error),

    #[error("Failed to execute request")]
    RequestExecutionFailed,

    #[error("File Error")]
    File(file::Error),

    #[error("Http Error")]
    Http(http::Error),

    #[error("Worker Error")]
    Worker(worker::Error),

    #[error("CLI Error")]
    CLI(CLIError),

    #[error("Inquire Error")]
    Inquire(InquireError),

    #[error("IRError {0}")]
    IRError(ir::Error),

    #[error("Serde URL Encoded Error")]
    SerdeUrlEncoded(serde_urlencoded::ser::Error),

    #[error("Hyper Header ToStr Error")]
    HyperHeaderToStr(hyper::header::ToStrError),

    #[error("Utf8 Error")]
    Utf8(Utf8Error),

    #[error("Rand Error")]
    Rand(rand::Error),
}

pub mod file {
    use std::string::FromUtf8Error;

    use derive_more::From;

    #[derive(From, thiserror::Error, Debug)]
    pub enum Error {
        #[error("No such file or directory (os error 2)")]
        NotFound,

        #[error("No permission to access the file")]
        NoPermission,

        #[error("Access denied")]
        AccessDenied,

        #[error("Invalid file format")]
        InvalidFormat,

        #[error("Invalid file path")]
        InvalidFilePath,

        #[error("Invalid OS string")]
        InvalidOsString,

        #[error("Failed to read file : {0}")]
        FileReadFailed(String),

        #[error("Failed to write file : {0}")]
        #[from(ignore)]
        FileWriteFailed(String),

        #[error("Std IO Error")]
        StdIO(std::io::Error),

        #[error("Utf8 Error")]
        Utf8(FromUtf8Error),

        #[error("File writing not supported on Lambda.")]
        LambdaFileWriteNotSupported,

        #[error("Cannot write to a file in an execution spec")]
        ExecutionSpecFileWriteFailed,

        #[error("Cloudflare Worker Execution Error : {0}")]
        #[from(ignore)]
        Cloudflare(String),
    }
}

pub mod http {
    use std::string::FromUtf8Error;

    use derive_more::From;

    #[derive(From, thiserror::Error, Debug)]
    pub enum Error {
        #[error("HTTP request failed with status code: {status_code}")]
        RequestFailed { status_code: u16 },

        #[error("Timeout occurred while making the HTTP request")]
        Timeout,

        #[error("Failed to parse the response body")]
        ResponseParse,

        #[error("Invalid URL: {url}")]
        InvalidUrl { url: String },

        #[error("Reqwest Middleware Error")]
        ReqwestMiddleware(reqwest_middleware::Error),

        #[error("Tonic Status Error")]
        TonicStatus(tonic::Status),

        #[error("Reqwest Error")]
        Reqwest(reqwest::Error),

        #[error("Serde Json Error")]
        SerdeJson(serde_json::Error),

        #[error("Unable to find key {0} in query params")]
        #[from(ignore)]
        KeyNotFound(String),

        #[error("Invalid Status Code")]
        InvalidStatusCode(hyper::http::status::InvalidStatusCode),

        #[error("Status Code error")]
        StatusCode,

        #[error("Invalid Header Value")]
        InvalidHeaderValue(hyper::header::InvalidHeaderValue),

        #[error("Invalid Header Name")]
        InvalidHeaderName(hyper::header::InvalidHeaderName),

        #[error("No mock found for request: {method} {url} in {spec_path}")]
        NoMockFound {
            method: String,
            url: String,
            spec_path: String,
        },

        #[error("Hyper HTTP Error")]
        Hyper(hyper::Error),

        #[error("Utf8 Error")]
        Utf8(FromUtf8Error),

        #[error("Invalid request host")]
        InvalidRequestHost,

        #[error("Hyper Http Error")]
        HyperHttp(hyper::http::Error),
    }
}

pub mod worker {
    use derive_more::From;

    #[derive(From, thiserror::Error, Debug)]
    pub enum Error {
        #[error("Failed to initialize worker")]
        InitializationFailed,

        #[error("Worker execution error")]
        ExecutionFailed,

        #[error("Worker communication error")]
        Communication,

        #[error("Serde Json Error")]
        SerdeJson(serde_json::Error),

        #[error("Request Clone Failed")]
        RequestCloneFailed,

        #[error("Hyper Header To Str Error")]
        HyperHeaderStr(hyper::header::ToStrError),

        #[error("CLI Error")]
        CLI(crate::cli::error::Error),

        #[error("JS Runtime Stopped Error")]
        JsRuntimeStopped,
    }
}

pub mod graphql {
    use derive_more::From;

    use super::http;

    #[derive(From, thiserror::Error, Debug)]
    pub enum Error {
        #[error("Serde Json Error")]
        SerdeJson(serde_json::Error),

        #[error("HTTP Error")]
        Http(http::Error),
    }
}

pub mod cache {
    use derive_more::From;

    #[derive(From, thiserror::Error, Debug)]
    pub enum Error {
        #[error("Serde Json Error")]
        SerdeJson(serde_json::Error),

        #[error("Worker Error : {0}")]
        Worker(String),

        #[error("Kv Error : {0}")]
        #[from(ignore)]
        Kv(String),
    }
}

pub type Result<A, E> = std::result::Result<A, E>;
