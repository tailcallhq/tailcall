use async_graphql::parser::types::Directive;
use derive_more::From;
use serde_json;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Type Mismatch: expected {expected}, but found {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Serde Json Error")]
    SerdeJsonError(serde_json::Error),

    #[error("{msg}: {directive:?}")]
    Missing { msg: String, directive: Directive },
}

pub type Result<A> = std::result::Result<A, Error>;
