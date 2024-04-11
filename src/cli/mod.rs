mod command;

mod fmt;
#[cfg(feature = "js")]
pub mod javascript;
pub mod metrics;
pub mod server;
mod tc;
pub mod telemetry;

pub mod runtime;
pub(crate) mod update_checker;

pub use tc::run;

use crate::error::Error;
impl From<rustls::Error> for Error {
    fn from(error: rustls::Error) -> Self {
        let cli_error = Error::new("Failed to create TLS Acceptor");
        let message = error.to_string();

        cli_error.description(message)
    }
}
