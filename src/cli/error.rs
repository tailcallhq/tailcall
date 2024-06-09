use crate::core::Errata;

impl From<rustls::Error> for Errata {
    fn from(error: rustls::Error) -> Self {
        let cli_error = Errata::new("Failed to create TLS Acceptor");
        let message = error.to_string();

        cli_error.description(message)
    }
}
