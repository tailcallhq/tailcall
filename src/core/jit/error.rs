use async_graphql::parser::types::OperationType;
use async_graphql::ErrorExtensions;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
#[error("Error while building the plan")]
pub enum BuildError {
    #[error("Root Operation type not defined for {operation}")]
    RootOperationTypeNotDefined { operation: OperationType },
    #[error("ResolveInputError: {0}")]
    ResolveInputError(#[from] ResolveInputError),
}

#[derive(Error, Debug, Clone)]
#[error("Cannot resolve the input value")]
pub enum ResolveInputError {
    #[error("Variable `{0}` is not defined")]
    VariableIsNotFound(String),
}

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("Build error: {0}")]
    BuildError(#[from] BuildError),
    #[error("ParseError: {0}")]
    ParseError(#[from] async_graphql::parser::Error),
    #[error(transparent)]
    IR(#[from] crate::core::ir::Error),
}

impl ErrorExtensions for Error {
    fn extend(&self) -> async_graphql::Error {
        match self {
            Error::BuildError(error) => error.extend(),
            Error::ParseError(error) => error.extend(),
            Error::IR(error) => error.extend(),
        }
    }
}

pub type Result<A> = std::result::Result<A, Error>;
