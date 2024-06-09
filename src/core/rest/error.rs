use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;

use async_graphql::parser::types::{Directive, Type};
use async_graphql::Name;
use derive_more::From;
use serde_json;

use crate::core::valid::ValidationError;

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

    #[error("Undefined query param: {0}")]
    UndefinedQueryParam(String),

    #[error("Parse Integer Error")]
    ParseIntegerError(ParseIntError),

    #[error("Parse Float Error")]
    ParseFloatingPointError(ParseFloatError),

    #[error("Parse Boolean Error")]
    ParseBooleanError(ParseBoolError),

    #[error("Undefined param : {key} in {input}")]
    UndefinedParam { key: String, input: String },

    #[error("Validation Error : {0}")]
    ValidationError(ValidationError<std::string::String>),

    #[error("Async Graphql Parser Error")]
    AsyncgraphqlParserError(async_graphql::parser::Error),

    #[error("Hyper HTTP Invalid URI Error")]
    HyperHttpInvalidUri(hyper::http::uri::InvalidUri),

    #[error("Hyper HTTP Error")]
    HyperHttpError(hyper::http::Error),

    #[error("Hyper Error")]
    HyperError(hyper::Error),

    #[error("Server Error")]
    #[from(ignore)]
    ServerError(String),
}

pub type Result<A> = std::result::Result<A, Error>;
