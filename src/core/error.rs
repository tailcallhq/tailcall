use derive_more::From;

// This is currently not getting used
#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Serde Json Error")]
    SerdeJsonError(serde_json::Error),
}

pub type Result<A, E> = std::result::Result<A, E>;
