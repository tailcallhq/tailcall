pub mod command;
mod error;
mod fmt;
#[cfg(feature = "js")]
pub mod javascript;
pub mod metrics;
pub mod server;
mod tc;
pub mod telemetry;

pub mod generator;

pub mod runtime;
pub(crate) mod update_checker;

pub use error::CLIError;
pub use tc::run::run;
