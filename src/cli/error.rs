use crate::core::Error;

impl From<rustls::Error> for Error {
    fn from(error: rustls::Error) -> Self {
        let cli_error = Error::new("Failed to create TLS Acceptor");
        let message = error.to_string();

        cli_error.description(message)
    }
}
