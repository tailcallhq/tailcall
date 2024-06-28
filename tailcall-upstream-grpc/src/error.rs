use std::env::VarError;

use derive_more::From;
use hyper::header::InvalidHeaderValue;
use opentelemetry::trace::TraceError;
use tracing::subscriber::SetGlobalDefaultError;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Hyper Error")]
    Hyper(hyper::Error),

    #[error("Set Global Default Error")]
    SetGlobalDefault(SetGlobalDefaultError),

    #[error("Trace Error")]
    Trace(TraceError),

    #[error("Failed to instantiate OTLP provider")]
    OltpProviderInstantiationFailed,

    #[error("Var Error")]
    Var(VarError),

    #[error("Invalid header value")]
    InvalidHeaderValue(InvalidHeaderValue),
}
