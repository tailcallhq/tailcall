use async_graphql::parser::types::{Directive, Type};
use async_graphql::Name;
use derive_more::From;
use serde_json;

#[derive(From, thiserror::Error, Debug)]
pub enum Error {
    #[error("Unexpected Named Type: {}", 0.to_string())]
    #[from(ignore)]
    UnexpectedNamedType(Name),

    #[error("Unexpected Type: {}, expected a named type like String, Float, Boolean etc.", 0.to_string())]
    UnexpectedType(Type),

    #[error("Serde Json Error")]
    SerdeJsonError(serde_json::Error),

    #[error("{msg}: {directive:?}")]
    Missing { msg: String, directive: Directive },

    #[error("Method not found")]
    MissingMethod,

    #[error("Path not found")]
    MissingPath,
}

pub type Result<A> = std::result::Result<A, Error>;
