use thiserror::Error;

#[derive(Error, Debug)]
#[error("BlueprintGenerationError: {0:?}")]
pub struct BlueprintGenerationError(pub crate::valid::ValidationError<String>);

impl From<crate::valid::ValidationError<String>> for BlueprintGenerationError {
    fn from(error: crate::valid::ValidationError<String>) -> Self {
        BlueprintGenerationError(error)
    }
}
