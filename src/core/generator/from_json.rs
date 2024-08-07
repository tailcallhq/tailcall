use std::collections::HashMap;

use convert_case::{Case, Casing};
use derive_getters::Getters;
use serde_json::Value;
use url::Url;

use super::json::{self, GraphQLTypesGenerator};
use super::NameGenerator;
use crate::core::config::{Config, GraphQLOperationType};
use crate::core::http::Method;
use crate::core::merge_right::MergeRight;
use crate::core::transform::{Transform, TransformerOps};
use crate::core::valid::{Valid, Validator};

#[derive(Getters)]
pub struct RequestSample {
    url: Url,
    method: Method,
    body: serde_json::Value,
    response: Value,
    field_name: String,
    operation_type: GraphQLOperationType,
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
            let (existing_name, suggested_name) = match sample.operation_type() {
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
                .pipe(json::SchemaGenerator::new(sample.operation_type()))
                .pipe(json::FieldBaseUrlGenerator::new(
                    sample.url(),
                    sample.operation_type(),
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
