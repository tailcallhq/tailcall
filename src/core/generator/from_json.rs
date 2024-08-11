use std::collections::HashMap;

use convert_case::{Case, Casing};
use serde_json::Value;
use url::Url;

use super::json::{self, GraphQLTypesGenerator};
use super::NameGenerator;
use crate::core::config::{Config, GraphQLOperationType};
use crate::core::http::Method;
use crate::core::merge_right::MergeRight;
use crate::core::transform::{Transform, TransformerOps};
use crate::core::valid::{Valid, Validator};

pub struct RequestSample {
    pub url: Url,
    pub method: Method,
    pub body: serde_json::Value,
    pub response: Value,
    pub field_name: String,
    pub operation_type: GraphQLOperationType,
}

impl RequestSample {
    #[allow(clippy::too_many_arguments)]
    pub fn new<T: Into<String>>(
        url: Url,
        method: Method,
        body: serde_json::Value,
        resp: Value,
        field_name: T,
        operation_type: GraphQLOperationType,
    ) -> Self {
        Self {
            url,
            method,
            body,
            response: resp,
            field_name: field_name.into(),
            operation_type,
        }
    }
}

pub struct FromJsonGenerator<'a> {
    request_samples: &'a [RequestSample],
    type_name_generator: &'a NameGenerator,
    query_name: &'a str,
    mutation_name: &'a Option<String>,
}

impl<'a> FromJsonGenerator<'a> {
    pub fn new(
        request_samples: &'a [RequestSample],
        type_name_generator: &'a NameGenerator,
        query_name: &'a str,
        mutation_name: &'a Option<String>,
    ) -> Self {
        Self {
            request_samples,
            type_name_generator,
            query_name,
            mutation_name,
        }
    }
}

impl Transform for FromJsonGenerator<'_> {
    type Value = Config;
    type Error = String;
    fn transform(&self, config: Self::Value) -> Valid<Self::Value, Self::Error> {
        let config_gen_req = self.request_samples;
        let type_name_gen = self.type_name_generator;

        Valid::from_iter(config_gen_req, |sample| {
            let (existing_name, suggested_name) = match sample.operation_type {
                GraphQLOperationType::Query => (
                    GraphQLOperationType::Query
                        .to_string()
                        .to_case(Case::Pascal),
                    self.query_name.to_owned(),
                ),
                GraphQLOperationType::Mutation => (
                    GraphQLOperationType::Mutation
                        .to_string()
                        .to_case(Case::Pascal),
                    self.mutation_name.clone().unwrap_or("Mutation".to_owned()),
                ),
            };

            let mut rename_types = HashMap::new();
            rename_types.insert(existing_name, suggested_name);

            // these transformations are required in order to generate a base config.
            GraphQLTypesGenerator::new(sample, type_name_gen)
                .pipe(json::SchemaGenerator::new(&sample.operation_type))
                .pipe(json::FieldBaseUrlGenerator::new(
                    &sample.url,
                    &sample.operation_type,
                ))
                .pipe(json::RenameTypes::new(rename_types))
                .transform(config.clone())
        })
        .map(|configs| {
            configs
                .iter()
                .fold(config, |acc, c| acc.merge_right(c.clone()))
        })
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::core::config::transformer::Preset;
    use crate::core::config::GraphQLOperationType;
    use crate::core::generator::{FromJsonGenerator, NameGenerator, RequestSample};
    use crate::core::http::Method;
    use crate::core::transform::TransformerOps;
    use crate::core::valid::Validator;

    #[derive(Deserialize)]
    struct JsonFixture {
        url: String,
        response: serde_json::Value,
    }

    fn parse_json(path: &str) -> JsonFixture {
        let content = std::fs::read_to_string(path).unwrap();
        let json_content: serde_json::Value = serde_json::from_str(&content).unwrap();

        JsonFixture {
            url: json_content["request"]["url"].as_str().unwrap().to_string(),
            response: json_content["response"]["body"].clone(),
        }
    }

    #[test]
    fn generate_config_from_json() -> anyhow::Result<()> {
        let mut request_samples = vec![];
        let fixtures = [
            "src/core/generator/tests/fixtures/json/incompatible_properties.json",
            "src/core/generator/tests/fixtures/json/list_incompatible_object.json",
            "src/core/generator/tests/fixtures/json/nested_list.json",
            "src/core/generator/tests/fixtures/json/nested_same_properties.json",
            "src/core/generator/tests/fixtures/json/incompatible_root_object.json",
        ];
        let field_name_generator = NameGenerator::new("f");
        for fixture in fixtures {
            let parsed_content = parse_json(fixture);
            request_samples.push(RequestSample::new(
                parsed_content.url.parse()?,
                Method::GET,
                serde_json::Value::Null,
                parsed_content.response,
                field_name_generator.next(),
                GraphQLOperationType::Query,
            ));
        }

        let config =
            FromJsonGenerator::new(&request_samples, &NameGenerator::new("T"), "Query", &None)
                .pipe(Preset::default())
                .generate()
                .to_result()?;

        insta::assert_snapshot!(config.to_sdl());
        Ok(())
    }
}
