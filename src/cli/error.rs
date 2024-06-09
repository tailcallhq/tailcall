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
    MetricsError(MetricsError),

    #[error("Rest Error")]
    RestError(rest::error::Error),

    #[error("Errata Error")]
    ErrataError(Errata),

    #[error("Serde Json Error")]
    SerdeJsonError(serde_json::Error),

    #[error("IO Error")]
    IOError(std::io::Error),

    #[error("Telemetry Trace Error : {0}")]
    TelemetryTraceError(String),

    #[error("Failed to send message")]
    MessageSendFailure,

    #[error("Hyper Error")]
    HyperError(hyper::Error),

    #[error("Rustls Error")]
    RustlsError(rustls::Error),

    #[error("Join Error")]
    JoinError(JoinError),

    #[error("Opentelemetry Global Error")]
    OpentelemetryGlobalError(opentelemetry::global::Error),

    #[error("Trace Error")]
    TraceError(TraceError),

    #[error("Log Error")]
    LogError(LogError),

    #[error("Utf8 Error")]
    Utf8Error(FromUtf8Error),

    #[error("Inquire Error")]
    InquireError(InquireError),

    #[error("Serde Yaml Error")]
    SerdeYamlError(serde_yaml::Error),

    #[error("Invalid Header Name")]
    InvalidHeaderName(hyper::header::InvalidHeaderName),

    #[error("Invalid Header Value")]
    InvalidHeaderValue(hyper::header::InvalidHeaderValue),

    #[error("rquickjs Error")]
    RQuickjsError(rquickjs::Error),

    #[error("Trying to reinitialize an already initialized QuickJS runtime")]
    ReinitializeQuickjsRuntimeError,

    #[error("Runtime not initialized")]
    RuntimeNotInitializedError,

    #[error("Deserialize Failed")]
    DeserializeFailed,

    #[error("Not a function error")]
    NotaFunctionError,

    #[error("Init Process Observer Error")]
    InitProcessObserverError,
}

pub type Result<A> = std::result::Result<A, Error>;

impl From<rustls::Error> for Errata {
    fn from(error: rustls::Error) -> Self {
        let cli_error = Errata::new("Failed to create TLS Acceptor");
        let message = error.to_string();

        cli_error.description(message)
    }
}
