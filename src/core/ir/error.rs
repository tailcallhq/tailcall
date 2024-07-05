use std::sync::Arc;

use async_graphql::{ErrorExtensions, Value as ConstValue};
use thiserror::Error;

use crate::core::auth;
#[derive(Debug, Error, Clone)]
pub enum Error {
    #[error("IOException: {0}")]
    IOException(String),

    #[error("gRPC Error: status: {grpc_code}, description: `{grpc_description}`, message: `{grpc_status_message}`")]
    GRPCError {
        grpc_code: i32,
        grpc_description: String,
        grpc_status_message: String,
        grpc_status_details: ConstValue,
    },

    #[error("APIValidationError: {0:?}")]
    APIValidationError(Vec<String>),

    #[error("ExprEvalError: {0}")]
    ExprEvalError(String),

    #[error("DeserializeError: {0}")]
    DeserializeError(String),

    #[error("Authentication Failure: {0}")]
    AuthError(auth::error::Error),
}

impl ErrorExtensions for Error {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self)).extend_with(|_err, e| {
            if let Error::GRPCError {
                grpc_code,
                grpc_description,
                grpc_status_message,
                grpc_status_details,
            } = self
            {
                e.set("grpcCode", *grpc_code);
                e.set("grpcDescription", grpc_description);
                e.set("grpcStatusMessage", grpc_status_message);
                e.set("grpcStatusDetails", grpc_status_details.clone());
            }
        })
    }
}

impl From<auth::error::Error> for Error {
    fn from(value: auth::error::Error) -> Self {
        Error::AuthError(value)
    }
}

impl<'a> From<crate::core::valid::ValidationError<&'a str>> for Error {
    fn from(value: crate::core::valid::ValidationError<&'a str>) -> Self {
        Error::APIValidationError(
            value
                .as_vec()
                .iter()
                .map(|e| e.message.to_owned())
                .collect(),
        )
    }
}

impl From<Arc<anyhow::Error>> for Error {
    fn from(error: Arc<anyhow::Error>) -> Self {
        match error.downcast_ref::<Error>() {
            Some(err) => err.clone(),
            None => Error::IOException(error.to_string()),
        }
    }
}

// TODO: remove conversion from anyhow and don't use anyhow to pass errors
// since it loses potentially valuable information that could be later provided
// in the error extensions
impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        match value.downcast::<Error>() {
            Ok(err) => err,
            Err(err) => Error::IOException(err.to_string()),
        }
    }
}
