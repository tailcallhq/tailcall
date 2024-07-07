use std::fmt::Display;
use std::str::Utf8Error;
use std::string::FromUtf8Error;

use derive_more::{From, DebugCustom};
use opentelemetry::metrics::MetricsError;
use opentelemetry::trace::TraceError;
use prost_reflect::DescriptorError;
use tokio::task::JoinError;

use super::config::UnsupportedConfigFormat;
use super::grpc::error::Error as GrpcError;
use super::ir;
use super::rest::error::Error as RestError;
use super::valid::ValidationError;

#[derive(From, DebugCustom)]
pub enum Error {
    #[debug(fmt = "Std IO Error")]
    StdIO(std::io::Error),

    #[debug(fmt = "Utf8 Error")]
    FromUtf8(FromUtf8Error),

    #[debug(fmt = "Validation Error : {}", _0)]
    Validation(ValidationError<std::string::String>),

    #[debug(fmt = "Serde Json Error")]
    SerdeJson(serde_json::Error),

    #[debug(fmt = "Serde Yaml Error")]
    SerdeYaml(serde_yaml::Error),

    #[debug(fmt = "Descriptor Error")]
    Descriptor(DescriptorError),

    #[debug(fmt = "Expected fully-qualified name for reference type but got {}", _0)]
    InvalidReferenceTypeName(String),

    #[debug(fmt = "Package name is required")]
    PackageNameNotFound,

    #[debug(fmt = "Protox Parse Error")]
    ProtoxParse(protox_parse::ParseError),

    #[debug(fmt = "URL Parse Error")]
    UrlParse(url::ParseError),

    #[debug(fmt = "Unable to extract content of google well-known proto file")]
    GoogleProtoFileContentNotExtracted,

    #[debug(fmt = "Unsupported Config Format")]
    UnsupportedConfigFormat(UnsupportedConfigFormat),

    #[debug(fmt = "Couldn't find definitions for service ServerReflection")]
    MissingServerReflectionDefinitions,

    #[debug(fmt = "Grpc Error")]
    Grpc(GrpcError),

    #[debug(fmt = "Serde Path To Error")]
    SerdePath(serde_path_to_error::Error<serde_json::Error>),

    #[debug(fmt = "Rest Error")]
    Rest(RestError),

    #[debug(fmt = "Expected fileDescriptorResponse but found none")]
    MissingFileDescriptorResponse,

    #[debug(fmt = "Prost Decode Error")]
    ProstDecode(prost::DecodeError),

    #[debug(fmt = "Received empty fileDescriptorProto")]
    EmptyFileDescriptorProto,

    #[debug(fmt = "Failed to decode fileDescriptorProto from BASE64")]
    FileDescriptorProtoDecodeFailed,

    #[debug(fmt = "Invalid header value")]
    HyperInvalidHeaderValue(hyper::header::InvalidHeaderValue),

    #[debug(fmt = "Invalid header name")]
    HyperInvalidHeaderName(hyper::header::InvalidHeaderName),

    #[debug(fmt = "Hyper HTTP Error")]
    HyperHttpError(hyper::http::Error),

    #[debug(fmt = "Prometheus Error")]
    Prometheus(prometheus::Error),

    #[debug(fmt = "Hyper Error")]
    Hyper(hyper::Error),

    #[debug(fmt = "Invalid URI")]
    InvalidUri(hyper::http::uri::InvalidUri),

    #[debug(fmt = "Mustache cannot be converted to const value")]
    InvalidMustacheConstConversion,

    #[debug(fmt = "Protox Error")]
    Protox(protox::Error),

    #[debug(fmt = "Failed to execute request")]
    RequestExecutionFailed,

    #[debug(fmt = "File Error: {}", _0)]
    File(file::Error),

    #[debug(fmt = "Http Error")]
    Http(http::Error),

    #[debug(fmt = "Worker Error")]
    Worker(worker::Error),

    #[debug(fmt = "IRError {}", _0)]
    IRError(ir::Error),

    #[debug(fmt = "Serde URL Encoded Error")]
    SerdeUrlEncoded(serde_urlencoded::ser::Error),

    #[debug(fmt = "Hyper Header ToStr Error")]
    HyperHeaderToStr(hyper::header::ToStrError),

    #[debug(fmt = "Utf8 Error")]
    Utf8(Utf8Error),

    #[debug(fmt = "Rand Error")]
    Rand(rand::Error),

    #[debug(fmt = "Trace Error")]
    Trace(TraceError),

    #[debug(fmt = "Join Error")]
    Join(JoinError),

    #[debug(fmt = "Metrics Error")]
    Metrics(MetricsError),

    #[debug(fmt = "Reqwest Error")]
    Reqwest(reqwest::Error),

    #[debug(fmt = "Unable to determine path")]
    PathDeterminationFailed,

    #[debug(fmt = "Schema mismatch Error")]
    SchemaMismatch,

    #[debug(fmt = "Failed to resolve parent value")]
    ParentValueNotResolved,

    #[debug(fmt = "Expected parent list index")]
    ExpectedParentListIndex,

    #[debug(fmt = "Can't resolve value for field")]
    FieldValueNotResolved,

    #[debug(fmt = "Expected list value")]
    ExpectedListValue,

    #[debug(fmt = "Headers Error")]
    Headers(headers::Error),

    #[debug(fmt = "Unsupported File Format")]
    UnsupportedFileFormat,

    #[debug(fmt = "Failed to match type_name")]
    TypenameMatchFailed,

    #[debug(fmt = "Value expected to be object")]
    ObjectExpected,

    #[debug(fmt = "Failed to find corresponding type for value")]
    MissingTypeForValue,

    #[debug(fmt = "CLI Error : {}", _0)]
    #[from(ignore)]
    CLI(String),

    #[debug(fmt = "Inquire Error : {}", _0)]
    #[from(ignore)]
    Inquire(String),
}

pub mod file {
    use std::string::FromUtf8Error;

    use derive_more::{From, DebugCustom};

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

    use derive_more::{From, DebugCustom};

    #[derive(From, DebugCustom)]
    pub enum Error {
        #[debug(fmt = "HTTP request failed with status code: {status_code}")]
        RequestFailed { status_code: u16 },

        #[debug(fmt = "Timeout occurred while making the HTTP request")]
        Timeout,

        #[debug(fmt = "Failed to parse the response body")]
        ResponseParse,

        #[debug(fmt = "Invalid URL: {url}")]
        InvalidUrl { url: String },

        #[debug(fmt = "Reqwest Middleware Error")]
        ReqwestMiddleware(reqwest_middleware::Error),

        #[debug(fmt = "Tonic Status Error")]
        TonicStatus(tonic::Status),

        #[debug(fmt = "Reqwest Error")]
        Reqwest(reqwest::Error),

        #[debug(fmt = "Serde Json Error")]
        SerdeJson(serde_json::Error),

        #[debug(fmt = "Unable to find key {} in query params", _0)]
        #[from(ignore)]
        KeyNotFound(String),

        #[debug(fmt = "Invalid Status Code")]
        InvalidStatusCode(hyper::http::status::InvalidStatusCode),

        #[debug(fmt = "Status Code error")]
        StatusCode,

        #[debug(fmt = "Invalid Header Value")]
        InvalidHeaderValue(hyper::header::InvalidHeaderValue),

        #[debug(fmt = "Invalid Header Name")]
        InvalidHeaderName(hyper::header::InvalidHeaderName),

        #[debug(fmt = "No mock found for request: {method} {url} in {spec_path}")]
        NoMockFound {
            method: String,
            url: String,
            spec_path: String,
        },

        #[debug(fmt = "Hyper HTTP Error")]
        Hyper(hyper::Error),

        #[debug(fmt = "Utf8 Error")]
        Utf8(FromUtf8Error),

        #[debug(fmt = "Invalid request host")]
        InvalidRequestHost,

        #[debug(fmt = "Hyper Http Error")]
        HyperHttp(hyper::http::Error),
    }
}

pub mod worker {
    use derive_more::{From, DebugCustom};

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
    use derive_more::{From, DebugCustom};

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
    use derive_more::{From, DebugCustom};

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

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for file::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for http::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for worker::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for graphql::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for cache::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

pub type Result<A, E> = std::result::Result<A, E>;
