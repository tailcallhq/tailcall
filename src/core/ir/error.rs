use std::fmt::Display;
use std::sync::Arc;

use async_graphql::{ErrorExtensions, Value as ConstValue};
use thiserror::Error;

use crate::core::auth;

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
        crate::core::Error::from(self.to_owned()).fmt(f)
    }
}

impl From<EvaluationError> for crate::core::Error {
    fn from(value: EvaluationError) -> Self {
        use crate::core::Error;
        match value {
            EvaluationError::IOException(message) => Error::new("IOException").description(message),
            EvaluationError::GRPCError {
                grpc_code,
                grpc_description,
                grpc_status_message,
                grpc_status_details,
            } => Error::new("GRPCError")
                .description(grpc_description)
                .caused_by(vec![Error::new(
                    format!("code: {}, message: {}", grpc_code, grpc_status_message).as_str(),
                )])
                .description(grpc_status_details.to_string()),
            EvaluationError::APIValidationError(errors) => Error::new("APIValidationError")
                .caused_by(errors.iter().map(|e| Error::new(e)).collect::<Vec<_>>()),
            EvaluationError::ExprEvalError(message) => {
                Error::new("ExprEvalError").description(message)
            }
            EvaluationError::DeserializeError(message) => {
                Error::new("DeserializeError").description(message)
            }

            EvaluationError::AuthError(message) => Error::new("AuthError").description(message),
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
