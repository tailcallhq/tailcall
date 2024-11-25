use async_graphql::dynamic::SchemaError;
use tailcall_valid::ValidationError;

use crate::core::Errata;

#[derive(Debug, thiserror::Error)]
pub enum BlueprintError {
    #[error("{0}")]
    Validation(String),

    #[error("{0}")]
    Validations(ValidationError<String>),

    #[error("{0}")]
    Discriminator(ValidationError<String>),

    #[error("{0}")]
    Mustache(ValidationError<String>),

    #[error("{0}")]
    JsonSchema(ValidationError<String>),

    #[error("{0}")]
    Directive(ValidationError<String>),

    #[error("{0}")]
    Grpc(ValidationError<String>),

    #[error(transparent)]
    Schema(#[from] SchemaError),

    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    #[error(transparent)]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),

    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),

    #[error(transparent)]
    Error(#[from] anyhow::Error),
}

impl PartialEq for BlueprintError {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl From<ValidationError<crate::core::blueprint::BlueprintError>> for Errata {
    fn from(error: ValidationError<crate::core::blueprint::BlueprintError>) -> Self {
        Errata::new("Blueprint Error").caused_by(
            error
                .as_vec()
                .iter()
                .map(|cause| {
                    let mut err =
                        Errata::new(&cause.message.to_string()).trace(cause.trace.clone().into());
                    if let Some(description) = &cause.description {
                        err = err.description(description.to_string());
                    }
                    err
                })
                .collect(),
        )
    }
}

impl From<&str> for BlueprintError {
    fn from(error: &str) -> Self {
        BlueprintError::Validation(error.to_string())
    }
}

impl From<String> for BlueprintError {
    fn from(error: String) -> Self {
        BlueprintError::Validation(error)
    }
}
