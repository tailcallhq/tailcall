use async_graphql::parser::types::OperationType;
use async_graphql::{ErrorExtensions, ServerError};
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
    ScalarInvalid { type_of: String, path: String },
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
    #[error("{0}")]
    ServerError(async_graphql::ServerError),
}

impl ErrorExtensions for Error {
    fn extend(&self) -> async_graphql::Error {
        match self {
            Error::BuildError(error) => error.extend(),
            Error::ParseError(error) => error.extend(),
            Error::IR(error) => error.extend(),
            Error::Validation(error) => error.extend(),
            Error::ServerError(error) => error.extend(),
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
