use std::env::VarError;

use derive_more::From;
use http::header::InvalidHeaderValue;
use opentelemetry::trace::TraceError;
use tracing::subscriber::SetGlobalDefaultError;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Hyper Error: {}", _0)]
    Hyper(hyper::Error),

    #[error("Set Global Default Error: {}", _0)]
    SetGlobalDefault(SetGlobalDefaultError),

    #[error("Trace Error: {}", _0)]
    Trace(TraceError),

    #[error("Failed to instantiate OTLP provider")]
    OltpProviderInstantiationFailed,

    #[error("Var Error: {}", _0)]
    Var(VarError),

    #[error("Invalid header value: {}", _0)]
    InvalidHeaderValue(InvalidHeaderValue),
}
