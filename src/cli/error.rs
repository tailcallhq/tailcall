use crate::core::CLIError;

impl From<rustls::Error> for CLIError {
    fn from(error: rustls::Error) -> Self {
        let cli_error = CLIError::new("Failed to create TLS Acceptor");
        let message = error.to_string();

        cli_error.description(message)
    }
}
