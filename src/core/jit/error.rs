use async_graphql::parser::types::OperationType;
use async_graphql::{ErrorExtensions, PathSegment, Pos, ServerError};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("Error while building the plan")]
pub enum BuildError {
    #[error("Root Operation type not defined for {operation}")]
    RootOperationTypeNotDefined { operation: OperationType },
    #[error("ResolveInputError: {0}")]
    ResolveInputError(#[from] ResolveInputError),
    #[error(r#"Unknown operation named "{0}""#)]
    OperationNotFound(String),
    #[error("Operation name required in request")]
    OperationNameRequired,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[error("Cannot resolve the input value")]
pub enum ResolveInputError {
    #[error("Variable `{0}` is not defined")]
    VariableIsNotFound(String),
}

#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    // TODO: replace with sane error message. Right now, it's defined as is only for compatibility
    // with async_graphql error message for this case
    #[error(r#"internal: invalid value for scalar "{type_of}", expected "FieldValue::Value""#)]
    ScalarInvalid { type_of: String },
    #[error(r#"internal: invalid item for enum "{type_of}""#)]
    EnumInvalid { type_of: String },
    #[error("internal: non-null types require a return value")]
    ValueRequired,
}

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("Build error: {0}")]
    BuildError(#[from] BuildError),
    #[error("ParseError: {0}")]
    ParseError(#[from] async_graphql::parser::Error),
    #[error(transparent)]
    IR(#[from] crate::core::ir::Error),
    #[error(transparent)]
    Validation(#[from] ValidationError),
}

impl ErrorExtensions for Error {
    fn extend(&self) -> async_graphql::Error {
        match self {
            Error::BuildError(error) => error.extend(),
            Error::ParseError(error) => error.extend(),
            Error::IR(error) => error.extend(),
            Error::Validation(error) => error.extend(),
        }
    }
}

pub type Result<A> = std::result::Result<A, Error>;

impl From<Error> for ServerError {
    fn from(val: Error) -> Self {
        // async_graphql::parser::Error has special conversion to ServerError
        if let Error::ParseError(error) = val {
            return error.into();
        }

        let extensions = val.extend().extensions;
        let mut server_error = ServerError::new(val.to_string(), None);

        server_error.extensions = extensions;

        server_error
    }
}

// impl From<Positioned<Error>> for Lift<ServerError> {
//     fn from(a: Positioned<Error>) -> Self {
//         (match a.node {
//             // async_graphql::parser::Error already has builtin positioning
//             Error::ParseError(error) => error.into(),
//             error => error.into_server_error_with_pos(Some(a.pos)),
//         })
//         .into()
//     }
// }

#[derive(Debug, Serialize)]
pub struct LocationError<Err> {
    pub error: Err,
    pub pos: Pos,
    pub path: Vec<PathSegment>,
}

impl From<LocationError<Error>> for ServerError {
    fn from(val: LocationError<Error>) -> Self {
        let extensions = val.error.extend().extensions;
        let mut server_error = ServerError::new(val.error.to_string(), Some(val.pos));

        server_error.extensions = extensions;

        // TODO: in order to be compatible with async_graphql path is only set for
        // validation errors here but in general we might consider setting it
        // for every error
        if let Error::Validation(_) = val.error {
            server_error.path = val.path;
        }

        server_error
    }
}
