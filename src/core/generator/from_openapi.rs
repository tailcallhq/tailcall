use oas3::{OpenApiV3Spec, Spec};

use crate::core::config::Config;
use crate::core::generator::json;
use crate::core::generator::openapi::QueryGenerator;
use crate::core::transform::{Transform, TransformerOps};
use crate::core::valid::{Valid, Validator};

pub struct FromOpenAPIGenerator {
    query: String,
    #[allow(unused)]
    spec: Spec,
}

impl FromOpenAPIGenerator {
    pub fn new(query: String, spec: OpenApiV3Spec) -> Self {
        Self { query, spec }
    }
}

impl Transform for FromOpenAPIGenerator {
    type Value = Config;
    type Error = String;

    fn transform(&self, value: Self::Value) -> Valid<Self::Value, Self::Error> {
        json::SchemaGenerator::new(self.query.clone())
            .pipe(QueryGenerator::new(self.query.as_str(), &self.spec))
            .transform(value)
    }
}

pub fn from_openapi_spec(query: &str, spec: OpenApiV3Spec) -> Config {
    let config = Config::default();
    let final_config = FromOpenAPIGenerator::new(query.to_string(), spec)
        .transform(config)
        .to_result();
    final_config.unwrap_or_else(|e| {
        tracing::warn!("Failed to generate config from OpenAPI spec: {}", e);
        Config::default()
    })
}
