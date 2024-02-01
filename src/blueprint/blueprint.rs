use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeSet, HashMap};
use std::num::NonZeroU64;

use async_graphql::dynamic::{Schema, SchemaBuilder};
use async_graphql::extensions::ApolloTracing;
use async_graphql::ValidationMode;
use derive_setters::Setters;
use serde_json::Value;

use super::GlobalTimeout;
use crate::blueprint::{Server, Upstream};
use crate::lambda::{Expression, Lambda};

/// Blueprint is an intermediary representation that allows us to generate graphQL APIs.
/// It can only be generated from a valid Config.
/// It allows us to choose a different GraphQL Backend, without re-writing all orchestration logic.
/// It's not optimized for REST APIs (yet).
#[derive(Clone, Debug, Default, Setters)]
pub struct Blueprint {
    pub definitions: Vec<Definition>,
    pub schema: SchemaDefinition,
    pub server: Server,
    pub upstream: Upstream,
}

#[derive(Clone, Debug)]
pub enum Type {
    NamedType { name: String, non_null: bool },
    ListType { of_type: Box<Type>, non_null: bool },
}

impl Default for Type {
    fn default() -> Self {
        Type::NamedType { name: "JSON".to_string(), non_null: false }
    }
}

impl Type {
    pub fn name(&self) -> &str {
        match self {
            Type::NamedType { name, .. } => name,
            Type::ListType { of_type, .. } => of_type.name(),
        }
    }
    /// checks if the type is nullable
    pub fn is_nullable(&self) -> bool {
        !match self {
            Type::NamedType { non_null, .. } => *non_null,
            Type::ListType { non_null, .. } => *non_null,
        }
    }
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct InterfaceTypeDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ObjectTypeDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
    pub description: Option<String>,
    pub implements: BTreeSet<String>,
}

#[derive(Clone, Debug)]
pub struct InputObjectTypeDefinition {
    pub name: String,
    pub fields: Vec<InputFieldDefinition>,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct EnumTypeDefinition {
    pub name: String,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
    pub enum_values: Vec<EnumValueDefinition>,
}

#[derive(Clone, Debug)]
pub struct EnumValueDefinition {
    pub description: Option<String>,
    pub name: String,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug, Default)]
pub struct SchemaDefinition {
    pub query: String,
    pub mutation: Option<String>,
    pub directives: Vec<Directive>,
}

#[derive(Clone, Debug)]
pub struct InputFieldDefinition {
    pub name: String,
    pub of_type: Type,
    pub default_value: Option<serde_json::Value>,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Cache {
    pub max_age: NonZeroU64,
    pub hasher: DefaultHasher,
}

#[derive(Clone, Debug, Setters, Default)]
pub struct FieldDefinition {
    pub name: String,
    pub args: Vec<InputFieldDefinition>,
    pub of_type: Type,
    pub resolver: Option<Expression>,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
    pub cache: Option<Cache>,
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

#[derive(Clone, Debug)]
pub struct Directive {
    pub name: String,
    pub arguments: HashMap<String, Value>,
    pub index: usize,
}

#[derive(Clone, Debug)]
pub struct ScalarTypeDefinition {
    pub name: String,
    pub directive: Vec<Directive>,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct UnionTypeDefinition {
    pub name: String,
    pub directives: Vec<Directive>,
    pub description: Option<String>,
    pub types: BTreeSet<String>,
}

///
/// Controls the kind of blueprint that is generated.
///
#[derive(Copy, Clone, Debug, Default)]
pub struct SchemaModifiers {
    /// If true, the generated schema will not have any resolvers.
    pub no_resolver: bool,
}

impl SchemaModifiers {
    pub fn no_resolver() -> Self {
        Self { no_resolver: true }
    }
}

impl Blueprint {
    pub fn query(&self) -> String {
        self.schema.query.clone()
    }

    pub fn mutation(&self) -> Option<String> {
        self.schema.mutation.clone()
    }

    fn drop_resolvers(mut self) -> Self {
        for def in self.definitions.iter_mut() {
            if let Definition::ObjectTypeDefinition(def) = def {
                for field in def.fields.iter_mut() {
                    field.resolver = None;
                }
            }
        }

        self
    }

    ///
    /// This function is used to generate a schema from a blueprint.
    ///
    pub fn to_schema(&self) -> Schema {
        self.to_schema_with(SchemaModifiers::default())
    }

    ///
    /// This function is used to generate a schema from a blueprint.
    /// The generated schema can be modified using the SchemaModifiers.
    ///
    pub fn to_schema_with(&self, schema_modifiers: SchemaModifiers) -> Schema {
        let blueprint = if schema_modifiers.no_resolver {
            self.clone().drop_resolvers()
        } else {
            self.clone()
        };

        let server = &blueprint.server;
        let mut schema = SchemaBuilder::from(&blueprint);

        if server.enable_apollo_tracing {
            schema = schema.extension(ApolloTracing);
        }

        if server.global_response_timeout > 0 {
            schema = schema
                .data(async_graphql::Value::from(server.global_response_timeout))
                .extension(GlobalTimeout);
        }

        if server.get_enable_query_validation() || schema_modifiers.no_resolver {
            schema = schema.validation_mode(ValidationMode::Strict);
        } else {
            schema = schema.validation_mode(ValidationMode::Fast);
        }

        if !server.get_enable_introspection() || schema_modifiers.no_resolver {
            schema = schema.disable_introspection();
        }

        // We should safely assume the blueprint is correct and,
        // generation of schema cannot fail.
        schema.finish().unwrap()
    }
}
