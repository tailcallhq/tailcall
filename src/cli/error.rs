use std::fmt::Display;

use crate::error::Error;
impl From<rustls::Error> for Error {
    fn from(error: rustls::Error) -> Self {
        Error::new("TLS Error").description(error.to_string())
    }
}

impl Display for crate::error::Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.pretty_print(f, true)
    }
}
