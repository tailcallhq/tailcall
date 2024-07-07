use std::fmt::Display;
use std::str::Utf8Error;
use std::string::FromUtf8Error;

use derive_more::From;
use opentelemetry::metrics::MetricsError;
use opentelemetry::trace::TraceError;
use prost_reflect::DescriptorError;
use tokio::task::JoinError;

use super::config::UnsupportedConfigFormat;
use super::grpc::error::Error as GrpcError;
use super::ir;
use super::rest::error::Error as RestError;
use super::valid::ValidationError;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Std IO Error")]
    StdIO(std::io::Error),

    #[error("Utf8 Error")]
    FromUtf8(FromUtf8Error),

    #[error("Validation Error : {}", _0)]
    Validation(ValidationError<std::string::String>),

    #[error("Serde Json Error")]
    SerdeJson(serde_json::Error),

    #[error("Serde Yaml Error")]
    SerdeYaml(serde_yaml::Error),

    #[error("Descriptor Error")]
    Descriptor(DescriptorError),

    #[error("Expected fully-qualified name for reference type but got {}", _0)]
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

    #[error("File Error: {}", _0)]
    File(file::Error),

    #[error("Http Error")]
    Http(http::Error),

    #[error("Worker Error")]
    Worker(worker::Error),

    #[error("IRError {}", _0)]
    IRError(ir::Error),

    #[error("Serde URL Encoded Error")]
    SerdeUrlEncoded(serde_urlencoded::ser::Error),

    #[error("Hyper Header ToStr Error")]
    HyperHeaderToStr(hyper::header::ToStrError),

    #[error("Utf8 Error")]
    Utf8(Utf8Error),

    #[error("Rand Error")]
    Rand(rand::Error),

    #[error("Trace Error")]
    Trace(TraceError),

    #[error("Join Error")]
    Join(JoinError),

    #[error("Metrics Error")]
    Metrics(MetricsError),

    #[error("Reqwest Error")]
    Reqwest(reqwest::Error),

    #[error("Unable to determine path")]
    PathDeterminationFailed,

    #[error("Schema mismatch Error")]
    SchemaMismatch,

    #[error("Failed to resolve parent value")]
    ParentValueNotResolved,

    #[error("Expected parent list index")]
    ExpectedParentListIndex,

    #[error("Can't resolve value for field")]
    FieldValueNotResolved,

    #[error("Expected list value")]
    ExpectedListValue,

    #[error("Headers Error")]
    Headers(headers::Error),

    #[error("Unsupported File Format")]
    UnsupportedFileFormat,

    #[error("Failed to match type_name")]
    TypenameMatchFailed,

    #[error("Value expected to be object")]
    ObjectExpected,

    #[error("Failed to find corresponding type for value")]
    MissingTypeForValue,

    #[error("CLI Error : {}", _0)]
    #[from(ignore)]
    CLI(String),

    #[error("Inquire Error : {}", _0)]
    #[from(ignore)]
    Inquire(String),
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

        #[debug(fmt = "Inquire Error : {}", _0)]
        #[from(ignore)]
        Inquire(String),

        #[debug(fmt = "Serde yaml Error")]
        SerdeYaml(serde_yaml::Error),
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

        #[error("Unable to find key {} in query params", _0)]
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
    }
}

pub mod graphql {
    use derive_more::{DebugCustom, From};

    use super::http;

    #[derive(From, DebugCustom)]
    pub enum Error {
        #[debug(fmt = "Serde Json Error")]
        SerdeJson(serde_json::Error),

        #[debug(fmt = "HTTP Error")]
        Http(http::Error),
    }
}

pub mod cache {
    use derive_more::{DebugCustom, From};

    #[derive(From, DebugCustom)]
    pub enum Error {
        #[debug(fmt = "Serde Json Error")]
        SerdeJson(serde_json::Error),

        #[debug(fmt = "Worker Error : {}", _0)]
        Worker(String),

        #[debug(fmt = "Kv Error : {}", _0)]
        #[from(ignore)]
        Kv(String),
    }
}

impl Display for file::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

impl Display for worker::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

impl Display for graphql::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

impl Display for cache::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

pub type Result<A, E> = std::result::Result<A, E>;
