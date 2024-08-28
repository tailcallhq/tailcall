use std::collections::{BTreeMap, BTreeSet, HashMap};

use convert_case::{Case, Casing};
use serde_json::Value;
use url::Url;

use super::json::{self, GraphQLTypesGenerator};
use super::{Input, NameGenerator};
use crate::core::config::transformer::RenameTypes;
use crate::core::config::{Config, GraphQLOperationType};
use crate::core::http::Method;
use crate::core::merge_right::MergeRight;
use crate::core::mustache::TemplateString;
use crate::core::transform::{Transform, TransformerOps};
use crate::core::valid::{Valid, Validator};

pub struct RequestSample {
    pub url: Url,
    pub method: Method,
    pub req_body: Value,
    pub res_body: Value,
    pub field_name: String,
    pub operation_type: GraphQLOperationType,
    pub headers: Option<BTreeMap<String, TemplateString>>,
}

impl From<&Input> for RequestSample {
    fn from(input: &Input) -> Self {
        match input {
            Input::Json {
                url,
                method,
                req_body,
                res_body,
                field_name,
                is_mutation,
                headers,
            } => {
                let operation_type = if *is_mutation {
                    GraphQLOperationType::Mutation
                } else {
                    GraphQLOperationType::Query
                };

                Self {
                    url: url.clone(),
                    method: method.clone(),
                    req_body: req_body.clone(),
                    res_body: res_body.clone(),
                    field_name: field_name.clone(),
                    headers: headers.clone(),
                    operation_type,
                }
            }
            _ => {
                panic!("Cannot convert from non-Json variant");
            }
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

            // collect the required header keys
            let header_keys = sample.headers.as_ref().map(|headers_inner| {
                headers_inner
                    .iter()
                    .map(|(k, _)| k.to_owned())
                    .collect::<BTreeSet<_>>()
            });

            let mut rename_types = HashMap::new();
            rename_types.insert(existing_name, suggested_name);

            // these transformations are required in order to generate a base config.
            GraphQLTypesGenerator::new(sample, type_name_gen)
                .pipe(json::SchemaGenerator::new(
                    &sample.operation_type,
                    &header_keys,
                ))
                .pipe(json::FieldBaseUrlGenerator::new(
                    &sample.url,
                    &sample.operation_type,
                ))
                .pipe(RenameTypes::new(rename_types.into_iter()))
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
    use crate::core::config::transformer::Preset;
    use crate::core::generator::generator::test::JsonFixture;
    use crate::core::generator::{FromJsonGenerator, Input, NameGenerator, RequestSample};
    use crate::core::transform::TransformerOps;
    use crate::core::valid::Validator;

    #[tokio::test]
    async fn generate_config_from_json() -> anyhow::Result<()> {
        let mut request_samples = vec![];
        let fixtures = [
            "src/core/generator/tests/fixtures/json/incompatible_properties.json",
            "src/core/generator/tests/fixtures/json/list_incompatible_object.json",
            "src/core/generator/tests/fixtures/json/nested_list.json",
            "src/core/generator/tests/fixtures/json/nested_same_properties.json",
            "src/core/generator/tests/fixtures/json/incompatible_root_object.json",
        ];
        for fixture in fixtures {
            let JsonFixture { request, response, is_mutation, field_name } =
                JsonFixture::read(fixture).await?;
            let json_input = Input::Json {
                url: request.url,
                method: request.method,
                req_body: request.body.unwrap_or_default(),
                res_body: response,
                field_name,
                is_mutation,
                headers: request.headers,
            };
            request_samples.push(RequestSample::from(&json_input));
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
