mod command;

pub mod error;
mod fmt;
#[cfg(feature = "js")]
pub mod javascript;
pub mod metrics;
pub mod runtime;
pub mod server;
mod tc;
pub mod telemetry;
pub(crate) mod update_checker;

pub use tc::run;
