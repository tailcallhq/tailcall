#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    #[error("Authentication failed: parameters not provided in the request.")]
    Missing,

    #[error("Authentication failed: {0}")]
    Parse(String),

    #[error("Authentication failed: Invalid credentials or token.")]
    Invalid,
}
