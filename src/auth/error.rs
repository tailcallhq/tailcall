#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    #[error("Parameters not provided in the request.")]
    Missing,

    #[error("{0}")]
    Parse(String),

    #[error("Invalid credentials or token.")]
    Invalid,
}
