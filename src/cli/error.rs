use std::string::FromUtf8Error;

use derive_more::From;
use inquire::InquireError;
use opentelemetry::logs::LogError;
use opentelemetry::metrics::MetricsError;
use opentelemetry::trace::TraceError;
use tokio::task::JoinError;

use crate::core::{rest, Errata};

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Metrics Error")]
    Metrics(MetricsError),

    #[error("Rest Error")]
    Rest(rest::error::Error),

    #[error("Errata Error")]
    Errata(Errata),

    #[error("Serde Json Error")]
    SerdeJson(serde_json::Error),

    #[error("IO Error")]
    IO(std::io::Error),

    #[error("Telemetry Trace Error : {0}")]
    TelemetryTrace(String),

    #[error("Failed to send message")]
    MessageSendFailure,

    #[error("Hyper Error")]
    Hyper(hyper::Error),

    #[error("Rustls Error")]
    Rustls(rustls::Error),

    #[error("Join Error")]
    Join(JoinError),

    #[error("Opentelemetry Global Error")]
    OpentelemetryGlobal(opentelemetry::global::Error),

    #[error("Trace Error")]
    Trace(TraceError),

    #[error("Log Error")]
    Log(LogError),

    #[error("Utf8 Error")]
    Utf8(FromUtf8Error),

    #[error("Inquire Error")]
    Inquire(InquireError),

    #[error("Serde Yaml Error")]
    SerdeYaml(serde_yaml::Error),

    #[error("Invalid Header Name")]
    InvalidHeaderName(hyper::header::InvalidHeaderName),

    #[error("Invalid Header Value")]
    InvalidHeaderValue(hyper::header::InvalidHeaderValue),

    #[error("rquickjs Error")]
    RQuickjs(rquickjs::Error),

    #[error("Trying to reinitialize an already initialized QuickJS runtime")]
    ReinitializeQuickjsRuntime,

    #[error("Runtime not initialized")]
    RuntimeNotInitialized,

    #[error("Deserialize Failed")]
    DeserializeFailed,

    #[error("Not a function error")]
    InvalidFunction,

    #[error("Init Process Observer Error")]
    InitProcessObserver,

    #[error("JS Runtime is stopped")]
    JsRuntimeStopped,

    #[error("Rustls internal error")]
    RustlsInternal,

    #[error("Reqwest middleware error")]
    ReqwestMiddleware(reqwest_middleware::Error),

    #[error("Reqwest error")]
    Reqwest(reqwest::Error),
}

pub type Result<A> = std::result::Result<A, Error>;

impl From<rustls::Error> for Errata {
    fn from(error: rustls::Error) -> Self {
        let cli_error = Errata::new("Failed to create TLS Acceptor");
        let message = error.to_string();

        cli_error.description(message)
    }
}
