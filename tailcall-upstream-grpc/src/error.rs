use std::env::VarError;
use std::fmt::Display;


use derive_more::{From, DebugCustom};
use hyper::header::InvalidHeaderValue;
use opentelemetry::trace::TraceError;
use tracing::subscriber::SetGlobalDefaultError;

#[derive(From, DebugCustom)]
pub enum Error {
    #[debug(fmt = "Hyper Error")]
    Hyper(hyper::Error),

    #[debug(fmt = "Set Global Default Error")]
    SetGlobalDefault(SetGlobalDefaultError),

    #[debug(fmt = "Trace Error")]
    Trace(TraceError),

    #[debug(fmt = "Failed to instantiate OTLP provider")]
    OltpProviderInstantiationFailed,

    #[debug(fmt = "Var Error")]
    Var(VarError),

    #[debug(fmt = "Invalid header value")]
    InvalidHeaderValue(InvalidHeaderValue),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
