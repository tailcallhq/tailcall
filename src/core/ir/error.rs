use std::fmt::Display;
use std::sync::Arc;

use async_graphql::{ErrorExtensions, Value as ConstValue};
use thiserror::Error;

use crate::core::{auth, Error};

#[derive(Debug, Error, Clone)]
pub enum EvaluationError {
    IOException(String),

    GRPCError {
        grpc_code: i32,
        grpc_description: String,
        grpc_status_message: String,
        grpc_status_details: ConstValue,
    },

    APIValidationError(Vec<String>),

    ExprEvalError(String),

    DeserializeError(String),

    AuthError(String),
}

impl Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::IOException(msg) => {
                write!(
                    f,
                    "{}",
                    Error::new("IO Exception").caused_by(vec![Error::new(msg)])
                )
            }
            EvaluationError::APIValidationError(errors) => {
                let cli_errors: Vec<Error> = errors.iter().map(|e| Error::new(e)).collect();
                write!(
                    f,
                    "{}",
                    Error::new("API Validation Error").caused_by(cli_errors)
                )
            }
            EvaluationError::ExprEvalError(msg) => write!(
                f,
                "{}",
                Error::new("Expr Eval Error").caused_by(vec![Error::new(msg)])
            ),
            EvaluationError::DeserializeError(msg) => write!(
                f,
                "{}",
                Error::new("Deserialize Error").caused_by(vec![Error::new(msg)])
            ),
            EvaluationError::AuthError(msg) => write!(
                f,
                "{}",
                Error::new("Authentication Failure").caused_by(vec![Error::new(msg)])
            ),
            EvaluationError::GRPCError {
                grpc_code,
                grpc_description,
                grpc_status_message,
                grpc_status_details: _,
            } => write!(
                f,
                "{}",
                Error::new("GRPC Error").caused_by(vec![
                    Error::new(format!("Status: {}", grpc_code).as_str()),
                    Error::new(format!("Message: {}", grpc_status_message).as_str()),
                    Error::new(format!("Description: {}", grpc_description).as_str())
                ])
            ),
        }
    }
}

impl ErrorExtensions for EvaluationError {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self)).extend_with(|_err, e| {
            if let EvaluationError::GRPCError {
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

impl From<auth::error::Error> for EvaluationError {
    fn from(value: auth::error::Error) -> Self {
        EvaluationError::AuthError(value.to_string())
    }
}

impl<'a> From<crate::core::valid::ValidationError<&'a str>> for EvaluationError {
    fn from(value: crate::core::valid::ValidationError<&'a str>) -> Self {
        EvaluationError::APIValidationError(
            value
                .as_vec()
                .iter()
                .map(|e| e.message.to_owned())
                .collect(),
        )
    }
}

impl From<Arc<anyhow::Error>> for EvaluationError {
    fn from(error: Arc<anyhow::Error>) -> Self {
        match error.downcast_ref::<EvaluationError>() {
            Some(err) => err.clone(),
            None => EvaluationError::IOException(error.to_string()),
        }
    }
}

// TODO: remove conversion from anyhow and don't use anyhow to pass errors
// since it loses potentially valuable information that could be later provided
// in the error extensions
impl From<anyhow::Error> for EvaluationError {
    fn from(value: anyhow::Error) -> Self {
        match value.downcast::<EvaluationError>() {
            Ok(err) => err,
            Err(err) => EvaluationError::IOException(err.to_string()),
        }
    }
}
