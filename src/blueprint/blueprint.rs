use std::collections::HashMap;

use async_graphql::dynamic::{Schema, SchemaBuilder};
use derive_setters::Setters;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use crate::config;

use crate::endpoint::Endpoint;

use crate::expression::Expression;
use crate::expression::Operation;
use crate::lambda::Lambda;

use async_graphql::extensions::ApolloTracing;
use async_graphql::*;

use super::GlobalTimeout;
use anyhow::Result;

/// Blueprint is an intermediary representation that allows us to generate graphQL APIs.
/// It can only be generated from a valid Config.
/// It allows us to choose a different GraphQL Backend, without re-writing all orchestration logic.
/// It's not optimized for REST APIs (yet).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Blueprint {
    pub definitions: Vec<Definition>,
    pub schema: SchemaDefinition,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Type {
    NamedType {
        name: String,
        #[serde(rename = "nonNull")]
        non_null: bool,
    },
    ListType {
        #[serde(rename = "ofType")]
        of_type: Box<Type>,
        #[serde(rename = "nonNull")]
        non_null: bool,
    },
}

impl Type {
    pub fn name(&self) -> &str {
        match self {
            Type::NamedType { name, .. } => name,
            Type::ListType { of_type, .. } => of_type.name(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Definition {
    InterfaceTypeDefinition(InterfaceTypeDefinition),
    ObjectTypeDefinition(ObjectTypeDefinition),
    InputObjectTypeDefinition(InputObjectTypeDefinition),
    ScalarTypeDefinition(ScalarTypeDefinition),
    EnumTypeDefinition(EnumTypeDefinition),
    UnionTypeDefinition(UnionTypeDefinition),
}
impl Definition {
    pub fn name(&self) -> &str {
        match self {
            Definition::InterfaceTypeDefinition(def) => &def.name,
            Definition::ObjectTypeDefinition(def) => &def.name,
            Definition::InputObjectTypeDefinition(def) => &def.name,
            Definition::ScalarTypeDefinition(def) => &def.name,
            Definition::EnumTypeDefinition(def) => &def.name,
            Definition::UnionTypeDefinition(def) => &def.name,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InterfaceTypeDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ObjectTypeDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
    pub description: Option<String>,
    pub implements: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputObjectTypeDefinition {
    pub name: String,
    pub fields: Vec<InputFieldDefinition>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EnumTypeDefinition {
    pub name: String,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
    pub enum_values: Vec<EnumValueDefinition>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EnumValueDefinition {
    pub description: Option<String>,
    pub name: String,
    pub directives: Vec<Directive>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SchemaDefinition {
    pub query: String,
    pub mutation: Option<String>,
    pub directives: Vec<Directive>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputFieldDefinition {
    pub name: String,
    #[serde(rename = "ofType")]
    pub of_type: Type,
    pub default_value: Option<serde_json::Value>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Setters)]
pub struct FieldDefinition {
    pub name: String,
    pub args: Vec<InputFieldDefinition>,
    #[serde(rename = "ofType")]
    pub of_type: Type,
    pub resolver: Option<Expression>,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
}

impl FieldDefinition {
    pub fn to_lambda(self) -> Option<Lambda<serde_json::Value>> {
        self.resolver.map(Lambda::new)
    }

    pub fn resolver_or_default(
        mut self,
        default_res: Lambda<serde_json::Value>,
        other: impl Fn(Lambda<serde_json::Value>) -> Lambda<serde_json::Value>,
    ) -> Self {
        self.resolver = match self.resolver {
            None => Some(default_res.expression),
            Some(expr) => Some(other(Lambda::new(expr)).expression),
        };
        self
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Directive {
    pub name: String,
    pub arguments: HashMap<String, Value>,
    pub index: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScalarTypeDefinition {
    pub name: String,
    pub directive: Vec<Directive>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnionTypeDefinition {
    pub name: String,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
    pub types: Vec<String>,
}
impl Blueprint {
    pub fn new(schema: SchemaDefinition, definitions: Vec<Definition>) -> Self {
        Self { schema, definitions }
    }

    pub fn query(&self) -> String {
        self.schema.query.clone()
    }

    pub fn mutation(&self) -> Option<String> {
        self.schema.mutation.clone()
    }

    pub fn to_schema(self, server: &config::Server) -> Result<Schema> {
        let mut schema = SchemaBuilder::from(self);

        if server.enable_apollo_tracing.unwrap_or(false) {
            schema = schema.extension(ApolloTracing);
        }

        let global_response_timeout = server.global_response_timeout.unwrap_or(0);
        if global_response_timeout > 0 {
            schema = schema
                .data(async_graphql::Value::from(global_response_timeout))
                .extension(GlobalTimeout);
        }

        if server.enable_query_validation() {
            schema = schema.validation_mode(ValidationMode::Strict);
        } else {
            schema = schema.validation_mode(ValidationMode::Fast);
        }
        if !server.enable_introspection() {
            schema = schema.disable_introspection();
        }
        Ok(schema.finish()?)
    }

    pub fn endpoints(&self) -> Vec<&Endpoint> {
        let mut endpoints = Vec::new();
        for definition in &self.definitions[..] {
            if let Definition::ObjectTypeDefinition(ObjectTypeDefinition { fields, .. }) = definition {
                for field in fields {
                    if let Some(Expression::Unsafe(_, Operation::Endpoint(endpoint), ..)) = &field.resolver {
                        endpoints.push(endpoint);
                    }
                }
            }
        }
        endpoints
    }
}

#[derive(Error, Debug)]
#[error("BlueprintGenerationError: {0:?}")]
pub struct BlueprintGenerationError(pub Vec<crate::cause::Cause<String>>);
