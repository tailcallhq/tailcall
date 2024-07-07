use std::fmt::Display;
use std::string::FromUtf8Error;

use derive_more::{From, DebugCustom};
use inquire::InquireError;
use opentelemetry::logs::LogError;
use opentelemetry::metrics::MetricsError;
use opentelemetry::trace::TraceError;
use tokio::task::JoinError;

use crate::core::valid::ValidationError;
use crate::core::{error, rest};

#[derive(From, DebugCustom)]
pub enum Error {
    #[debug(fmt = "Metrics Error")]
    Metrics(MetricsError),

    #[debug(fmt = "Rest Error")]
    Rest(rest::error::Error),

    #[debug(fmt = "Serde Json Error")]
    SerdeJson(serde_json::Error),

    #[debug(fmt = "IO Error")]
    IO(std::io::Error),

    #[debug(fmt = "Telemetry Trace Error : {}", _0)]
    TelemetryTrace(String),

    #[debug(fmt = "Failed to send message")]
    MessageSendFailure,

    #[debug(fmt = "Hyper Error")]
    Hyper(hyper::Error),

    #[debug(fmt = "Rustls Error")]
    Rustls(rustls::Error),

    #[debug(fmt = "Join Error")]
    Join(JoinError),

    #[debug(fmt = "Opentelemetry Global Error")]
    OpentelemetryGlobal(opentelemetry::global::Error),

    #[debug(fmt = "Trace Error")]
    Trace(TraceError),

    #[debug(fmt = "Log Error")]
    Log(LogError),

    #[debug(fmt = "Utf8 Error")]
    Utf8(FromUtf8Error),

    #[debug(fmt = "Inquire Error")]
    Inquire(InquireError),

    #[debug(fmt = "Serde Yaml Error")]
    SerdeYaml(serde_yaml::Error),

    #[debug(fmt = "Invalid Header Name")]
    InvalidHeaderName(hyper::header::InvalidHeaderName),

    #[debug(fmt = "Invalid Header Value")]
    InvalidHeaderValue(hyper::header::InvalidHeaderValue),

    #[debug(fmt = "rquickjs Error")]
    RQuickjs(rquickjs::Error),

    #[debug(fmt = "Trying to reinitialize an already initialized QuickJS runtime")]
    ReinitializeQuickjsRuntime,

    #[debug(fmt = "Runtime not initialized")]
    RuntimeNotInitialized,

    #[debug(fmt = "Deserialize Failed")]
    DeserializeFailed,

    #[debug(fmt = "Not a function error")]
    InvalidFunction,

    #[debug(fmt = "Init Process Observer Error")]
    InitProcessObserver,

    #[debug(fmt = "JS Runtime is stopped")]
    JsRuntimeStopped,

    #[debug(fmt = "Rustls internal error")]
    RustlsInternal,

    #[debug(fmt = "Reqwest middleware error")]
    ReqwestMiddleware(reqwest_middleware::Error),

    #[debug(fmt = "Reqwest error")]
    Reqwest(reqwest::Error),

    #[debug(fmt = "Validation Error : {}", _0)]
    Validation(ValidationError<std::string::String>),

    #[debug(fmt = "Core Error")]
    CoreError(error::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Result<A> = std::result::Result<A, Error>;
