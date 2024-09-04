use derive_setters::Setters;

use super::infer_arg_name::InferArgName;
use super::infer_field_name::InferFieldName;
use super::Error;
use crate::cli::llm::InferTypeName;
use crate::core::config::Config;
use crate::core::{AsyncTransform, AsyncTransformerOps};

/// Defines a set of default transformers that can be applied to any
/// configuration to make it more maintainable and readable.
#[derive(Setters, Debug, PartialEq)]
pub struct AsyncPreset {
    pub model: String,
    pub secret: Option<String>,
}

impl AsyncPreset {
    pub fn new(model: String, secret: Option<String>) -> Self {
        Self { model, secret }
    }
}

impl AsyncTransform for AsyncPreset {
    type Value = Config;
    type Error = Error;

    async fn transform(
        &self,
        value: Self::Value,
    ) -> crate::core::valid::Valid<Self::Value, Self::Error> {
        let model = self.model.clone();
        let secret = self.secret.clone();

        // Note: try to keep the order same.
        InferTypeName::new(model.clone(), secret.clone())
            .pipe(InferFieldName::new(model.clone(), secret.clone()))
            .pipe(InferArgName::new(model, secret))
            .transform(value)
            .await
    }
}
