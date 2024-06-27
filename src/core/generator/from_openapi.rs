use oas3::{OpenApiV3Spec, Spec};

use crate::core::config::Config;
use crate::core::generator::json;
use crate::core::transform::Transform;
use crate::core::valid::{Valid, Validator};

#[derive(Default)]
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
        json::SchemaGenerator::new(self.query.clone()).transform(value)
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_openapi_apis_guru() {
        let apis_guru = config_from_openapi_spec("apis-guru.yml");
        insta::assert_snapshot!(apis_guru);
    }

    #[test]
    fn test_openapi_jsonplaceholder() {
        let jsonplaceholder = config_from_openapi_spec("jsonplaceholder.yml");
        insta::assert_snapshot!(jsonplaceholder);
    }

    #[test]
    fn test_openapi_spotify() {
        let spotify = config_from_openapi_spec("spotify.yml");
        insta::assert_snapshot!(spotify);
    }

    fn config_from_openapi_spec(filename: &str) -> String {
        let spec_path = Path::new("src")
            .join("core")
            .join("generator")
            .join("tests")
            .join("fixtures")
            .join("openapi")
            .join(filename);

        let spec = oas3::from_path(spec_path).unwrap();
        from_openapi_spec("Query", spec).to_sdl()
    }
}
