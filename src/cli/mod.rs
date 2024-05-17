mod command;
mod error;
// TODO: make it private after testing.
pub mod fmt;
#[cfg(feature = "js")]
pub mod javascript;
pub mod metrics;
pub mod server;
mod tc;
pub mod telemetry;

pub mod runtime;
pub(crate) mod update_checker;

pub use error::CLIError;
pub use tc::run;
