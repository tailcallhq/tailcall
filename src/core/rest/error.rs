use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;

use async_graphql::parser::types::{Directive, Type};
use async_graphql::Name;
use derive_more::{DebugCustom, From};
use serde_json;

use crate::core::valid::ValidationError;
use std::fmt::Display;


#[derive(From, DebugCustom)]
pub enum Error {
    #[debug(fmt = "Unexpected Named Type: {}", 0.to_string())]
    #[from(ignore)]
    UnexpectedNamedType(Name),

    #[debug(fmt = "Unexpected Type: {}, expected a named type like String, Float, Boolean etc.", 0.to_string())]
    UnexpectedType(Type),

    #[debug(fmt = "Serde Json Error")]
    SerdeJsonError(serde_json::Error),

    #[debug(fmt = "{msg}: {directive:?}")]
    Missing { msg: String, directive: Directive },

    #[debug(fmt = "Method not provided in the directive")]
    MissingMethod,

    #[debug(fmt = "Path not provided in the directive")]
    MissingPath,

    #[debug(fmt = "Undefined query param: {}", _0)]
    UndefinedQueryParam(String),

    #[debug(fmt = "Parse Integer Error")]
    ParseIntegerError(ParseIntError),

    #[debug(fmt = "Parse Float Error")]
    ParseFloatingPointError(ParseFloatError),

    #[debug(fmt = "Parse Boolean Error")]
    ParseBooleanError(ParseBoolError),

    #[debug(fmt = "Undefined param : {key} in {input}")]
    UndefinedParam { key: String, input: String },

    #[debug(fmt = "Validation Error : {}", _0)]
    ValidationError(ValidationError<std::string::String>),

    #[debug(fmt = "Async Graphql Parser Error")]
    AsyncgraphqlParserError(async_graphql::parser::Error),

    #[debug(fmt = "Hyper HTTP Invalid URI Error")]
    HyperHttpInvalidUri(hyper::http::uri::InvalidUri),

    #[debug(fmt = "Hyper HTTP Error")]
    HyperHttpError(hyper::http::Error),

    #[debug(fmt = "Hyper Error")]
    HyperError(hyper::Error),

    #[debug(fmt = "Server Error")]
    #[from(ignore)]
    ServerError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Result<A> = std::result::Result<A, Error>;
