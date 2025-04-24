use std::fmt::Display;
use std::sync::Arc;

use async_graphql::Value as ConstValue;
use derive_more::From;
use thiserror::Error;

use crate::core::jit::graphql_error::{Error as ExtensionError, ErrorExtensions};
use crate::core::{auth, cache, worker, Errata};

#[derive(From, Debug, Error, Clone)]
pub enum Error {
    IO(String),
    HTTP {
        message: String,
        body: String,
    },
    GRPC {
        grpc_code: i32,
        grpc_description: String,
        grpc_status_message: String,
        grpc_status_details: ConstValue,
    },

    APIValidation(Vec<String>),

    #[from(ignore)]
    ExprEval(String),

    #[from(ignore)]
    Deserialize(String),

    Auth(auth::error::Error),

    Worker(worker::Error),

    Cache(cache::Error),

    #[from(ignore)]
    Entity(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Errata::from(self.to_owned()).fmt(f)
    }
}

impl From<Error> for Errata {
    fn from(value: Error) -> Self {
        match value {
            Error::IO(message) => Errata::new("IOException").description(message),
            Error::HTTP{ message, body:_ } => Errata::new("HTTP Error")
                .description(message),
            Error::GRPC {
                grpc_code,
                grpc_description,
                grpc_status_message,
                grpc_status_details: _,
            } => Errata::new("gRPC Error")
                .description(format!("status: {grpc_code}, description: `{grpc_description}`, message: `{grpc_status_message}`")),
            Error::APIValidation(errors) => Errata::new("API Validation Error")
                .caused_by(errors.iter().map(|e| Errata::new(e)).collect::<Vec<_>>()),
            Error::Deserialize(message) => {
                Errata::new("Deserialization Error").description(message)
            }
            Error::ExprEval(message) => {
                Errata::new("Expression Evaluation Error").description(message)
            }
            Error::Auth(err) => {
                Errata::new("Authentication Failure").description(err.to_string())
            }
            Error::Worker(err) => Errata::new("Worker Error").description(err.to_string()),
            Error::Cache(err) => Errata::new("Cache Error").description(err.to_string()),
            Error::Entity(message) => Errata::new("Entity Resolver Error").description(message)
        }
    }
}

impl ErrorExtensions for Error {
    fn extend(&self) -> ExtensionError {
        ExtensionError::new(format!("{}", self)).extend_with(|_err, e| {
            if let Error::GRPC {
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

            if let Error::HTTP { message: _, body } = self {
                if let Ok(ConstValue::Object(map)) = serde_json::from_str::<ConstValue>(body) {
                    e.extend(map);
                } else {
                    e.set("cause", body);
                }
            }
        })
    }
}

impl<'a> From<tailcall_valid::ValidationError<&'a str>> for Error {
    fn from(value: tailcall_valid::ValidationError<&'a str>) -> Self {
        Error::APIValidation(
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
            None => Error::IO(error.to_string()),
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
            Err(err) => Error::IO(err.to_string()),
        }
    }
}
