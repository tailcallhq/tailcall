use std::collections::HashSet;

use anyhow::Result;
use async_graphql::parser::types::ServiceDocument;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use tailcall_typedefs_common::directive_definition::DirectiveDefinition;
use tailcall_typedefs_common::input_definition::InputDefinition;
use tailcall_typedefs_common::ServiceDocumentBuilder;
use tailcall_valid::{Valid, Validator};

use super::from_document::from_document;
use super::{AddField, Alias, Cache, Call, Discriminate, Expr, GraphQL, Grpc, Http, Link, Modify, Omit, Protected, Server, Telemetry, Upstream, JS, SchemaConfig};
use crate::core::config::runtime_config::RuntimeConfig;
use crate::core::config::source::Source;
use crate::core::merge_right::MergeRight;
use crate::core::scalar::Scalar;


#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    schemars::JsonSchema,
)]
pub struct Config {
    pub schema_config: SchemaConfig,
    pub runtime_config: RuntimeConfig,
}

impl MergeRight for Config {
    fn merge_right(mut self, other: Self) -> Self {
        self.schema_config = self.schema_config.merge_right(other.schema_config);

        self
    }
}

impl Config {

    pub fn schema_config(&self) -> &SchemaConfig {
        &self.schema_config
    }
    pub fn runtime_config(&self) -> &RuntimeConfig {
        &self.runtime_config
    }

    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(yaml)?)
    }

    pub fn from_sdl(sdl: &str) -> Valid<Self, String> {
        let doc = async_graphql::parser::parse_schema(sdl);
        match doc {
            Ok(doc) => from_document(doc),
            Err(e) => Valid::fail(e.to_string()),
        }
    }

    pub fn from_source(source: Source, schema: &str) -> Result<Self> {
        match source {
            Source::GraphQL => Ok(Config::from_sdl(schema).to_result()?),
            Source::Json => Ok(Config::from_json(schema)?),
            Source::Yml => Ok(Config::from_yaml(schema)?),
        }
    }

    pub fn graphql_schema() -> ServiceDocument {
        // Multiple structs may contain a field of the same type when creating directive
        // definitions. To avoid generating the same GraphQL type multiple times,
        // this hash set is used to track visited types and ensure no duplicates are
        // generated.
        let mut generated_types: HashSet<String> = HashSet::new();
        let generated_types = &mut generated_types;

        let builder = ServiceDocumentBuilder::new();
        let mut builder = builder
            .add_directive(AddField::directive_definition(generated_types))
            .add_directive(Alias::directive_definition(generated_types))
            .add_directive(Cache::directive_definition(generated_types))
            .add_directive(Call::directive_definition(generated_types))
            .add_directive(Expr::directive_definition(generated_types))
            .add_directive(GraphQL::directive_definition(generated_types))
            .add_directive(Grpc::directive_definition(generated_types))
            .add_directive(Http::directive_definition(generated_types))
            .add_directive(JS::directive_definition(generated_types))
            .add_directive(Link::directive_definition(generated_types))
            .add_directive(Modify::directive_definition(generated_types))
            .add_directive(Omit::directive_definition(generated_types))
            .add_directive(Protected::directive_definition(generated_types))
            .add_directive(Server::directive_definition(generated_types))
            .add_directive(Telemetry::directive_definition(generated_types))
            .add_directive(Upstream::directive_definition(generated_types))
            .add_directive(Discriminate::directive_definition(generated_types))
            .add_input(GraphQL::input_definition())
            .add_input(Grpc::input_definition())
            .add_input(Http::input_definition())
            .add_input(Expr::input_definition())
            .add_input(JS::input_definition())
            .add_input(Modify::input_definition())
            .add_input(Cache::input_definition())
            .add_input(Telemetry::input_definition());

        for scalar in Scalar::iter() {
            builder = builder.add_scalar(scalar.scalar_definition());
        }

        builder.build()
    }
}

#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default, schemars::JsonSchema,
)]
pub enum Encoding {
    #[default]
    ApplicationJson,
    ApplicationXWwwFormUrlencoded,
}
