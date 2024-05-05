#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    #[error("Missing Authorization Header")]
    Missing,

    #[error("{0}")]
    Parse(String),

    #[error("Invalid Authorization Header")]
    Invalid,
}
