use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;

use async_graphql::parser::types::{Directive, Type};
use async_graphql::{Name, ServerError};
use derive_more::{DebugCustom, From};
use serde_json;

use crate::core::valid::ValidationError;

#[derive(From, thiserror::Error, DebugCustom)]
pub enum Error {
    #[error("Unexpected Named Type: {}", 0.to_string())]
    UnexpectedNamedType(Name),

    #[error("Unexpected Type: {}, expected a named type like String, Float, Boolean etc.", 0.to_string())]
    UnexpectedType(Type),

    #[error("Serde Json Error: {}", _0)]
    SerdeJson(serde_json::Error),

    #[error("{msg}: {directive:?}")]
    #[debug(fmt = "{msg}: {directive:?}")]
    Missing { msg: String, directive: Directive },

    #[error("Undefined query param: {}", _0)]
    UndefinedQueryParam(String),

    #[error("Parse Integer Error: {}", _0)]
    ParseInteger(ParseIntError),

    #[error("Parse Float Error: {}", _0)]
    ParseFloatingPoint(ParseFloatError),

    #[error("Parse Boolean Error: {}", _0)]
    ParseBoolean(ParseBoolError),

    #[error("Undefined param : {key} in {input}")]
    #[debug(fmt = "Undefined param : {key} in {input}")]
    UndefinedParam { key: String, input: String },

    #[error("Validation Error : {}", _0)]
    Validation(ValidationError<std::string::String>),

    #[error("Async Graphql Parser Error: {}", _0)]
    AsyncgraphqlParser(async_graphql::parser::Error),

    #[error("Hyper HTTP Invalid URI Error: {}", _0)]
    HyperHttpInvalidUri(hyper::http::uri::InvalidUri),

    #[error("Hyper HTTP Error: {}", _0)]
    HyperHttp(hyper::http::Error),

    #[error("Hyper Error: {}", _0)]
    Hyper(hyper::Error),

    #[error("Async Graphql Server Error: {}", _0)]
    AsyncgraphqlServer(ServerError),
}

pub type Result<A> = std::result::Result<A, Error>;
