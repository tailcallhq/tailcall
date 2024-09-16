use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::{self, Display};
use std::num::NonZeroU64;

use anyhow::Result;
use async_graphql::parser::types::{ConstDirective, ServiceDocument};
use async_graphql::Positioned;
use derive_setters::Setters;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall_macros::{CustomResolver, DirectiveDefinition, InputDefinition};
use tailcall_typedefs_common::directive_definition::DirectiveDefinition;
use tailcall_typedefs_common::input_definition::InputDefinition;
use tailcall_typedefs_common::ServiceDocumentBuilder;

use super::directives::{Call, EntityResolver, Expr, GraphQL, Grpc, Http, Key, JS};
use super::from_document::from_document;
use super::telemetry::Telemetry;
use super::{Link, Server, Upstream};
use crate::core::config::npo::QueryPath;
use crate::core::config::source::Source;
use crate::core::directive::DirectiveCodec;
use crate::core::is_default;
use crate::core::macros::MergeRight;
use crate::core::merge_right::MergeRight;
use crate::core::scalar::Scalar;
use crate::core::valid::{Valid, Validator};

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    Setters,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    MergeRight,
)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    ///
    /// Dictates how the server behaves and helps tune tailcall for all ingress
    /// requests. Features such as request batching, SSL, HTTP2 etc. can be
    /// configured here.
    #[serde(default)]
    pub server: Server,

    ///
    /// Dictates how tailcall should handle upstream requests/responses.
    /// Tuning upstream can improve performance and reliability for connections.
    #[serde(default)]
    pub upstream: Upstream,

    ///
    /// Specifies the entry points for query and mutation in the generated
    /// GraphQL schema.
    pub schema: RootSchema,

    ///
    /// A map of all the types in the schema.
    #[serde(default)]
    #[setters(skip)]
    pub types: BTreeMap<String, Type>,

    ///
    /// A map of all the union types in the schema.
    #[serde(default, skip_serializing_if = "is_default")]
    pub unions: BTreeMap<String, Union>,

    ///
    /// A map of all the enum types in the schema
    #[serde(default, skip_serializing_if = "is_default")]
    pub enums: BTreeMap<String, Enum>,

    ///
    /// A list of all links in the schema.
    #[serde(default, skip_serializing_if = "is_default")]
    pub links: Vec<Link>,
    #[serde(default, skip_serializing_if = "is_default")]
    /// Enable [opentelemetry](https://opentelemetry.io) support
    pub telemetry: Telemetry,
}

///
/// Represents a GraphQL type.
/// A type can be an object, interface, enum or scalar.
#[derive(
    Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema, MergeRight,
)]
pub struct Type {
    ///
    /// A map of field name and its definition.
    pub fields: BTreeMap<String, Field>,
    #[serde(default, skip_serializing_if = "is_default")]
    ///
    /// Additional fields to be added to the type
    pub added_fields: Vec<AddField>,
    #[serde(default, skip_serializing_if = "is_default")]
    ///
    /// Documentation for the type that is publicly visible.
    pub doc: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    ///
    /// Interfaces that the type implements.
    pub implements: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    ///
    /// Setting to indicate if the type can be cached.
    pub cache: Option<Cache>,
    ///
    /// Marks field as protected by auth providers
    #[serde(default)]
    pub protected: Option<Protected>,

    ///
    /// Apollo federation entity resolver.
    #[serde(flatten, default, skip_serializing_if = "is_default")]
    pub resolver: Option<Resolver>,

    ///
    /// Apollo federation key directive.
    /// skip since it's set automatically by config transformer
    #[serde(skip_serializing)]
    pub key: Option<Key>,
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;

        for (field_name, field) in &self.fields {
            writeln!(f, "  {}: {:?},", field_name, field.type_of)?;
        }
        writeln!(f, "}}")
    }
}

impl Type {
    pub fn fields(mut self, fields: Vec<(&str, Field)>) -> Self {
        let mut graphql_fields = BTreeMap::new();
        for (name, field) in fields {
            graphql_fields.insert(name.to_string(), field);
        }
        self.fields = graphql_fields;
        self
    }

    pub fn scalar(&self) -> bool {
        self.fields.is_empty()
    }
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Deserialize,
    Serialize,
    Eq,
    schemars::JsonSchema,
    MergeRight,
    DirectiveDefinition,
    InputDefinition,
)]
#[directive_definition(locations = "Object,FieldDefinition")]
/// The @cache operator enables caching for the query, field or type it is
/// applied to.
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Cache {
    /// Specifies the duration, in milliseconds, of how long the value has to be
    /// stored in the cache.
    pub max_age: NonZeroU64,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    Default,
    schemars::JsonSchema,
    MergeRight,
    DirectiveDefinition,
)]
#[directive_definition(locations = "Object,FieldDefinition")]
pub struct Protected {}

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    Setters,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    MergeRight,
)]
#[setters(strip_option)]
pub struct RootSchema {
    pub query: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub mutation: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub subscription: Option<String>,
}

#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema, DirectiveDefinition,
)]
#[directive_definition(locations = "FieldDefinition")]
#[serde(deny_unknown_fields)]
/// Used to omit a field from public consumption.
pub struct Omit {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApolloFederation {
    EntityResolver(EntityResolver),
    Service,
}

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    MergeRight,
    CustomResolver,
)]
#[serde(rename_all = "camelCase")]
pub enum Resolver {
    Http(Http),
    Grpc(Grpc),
    Graphql(GraphQL),
    Call(Call),
    Js(JS),
    Expr(Expr),
    #[serde(skip)]
    #[resolver(skip_directive)]
    ApolloFederation(ApolloFederation),
}

impl Resolver {
    pub fn is_batched(&self) -> bool {
        match self {
            Resolver::Http(http) => !http.batch_key.is_empty(),
            Resolver::Grpc(grpc) => !grpc.batch_key.is_empty(),
            Resolver::Graphql(graphql) => graphql.batch,
            Resolver::ApolloFederation(ApolloFederation::EntityResolver(entity_resolver)) => {
                entity_resolver
                    .resolver_by_type
                    .values()
                    .any(Resolver::is_batched)
            }
            _ => false,
        }
    }
}

///
/// A field definition containing all the metadata information about resolving a
/// field.
#[derive(
    Serialize, Deserialize, Clone, Debug, Default, Setters, PartialEq, Eq, schemars::JsonSchema,
)]
#[setters(strip_option)]
pub struct Field {
    ///
    /// Refers to the type of the value the field can be resolved to.
    #[serde(rename = "type", default, skip_serializing_if = "is_default")]
    pub type_of: crate::core::Type,

    ///
    /// Map of argument name and its definition.
    #[serde(default, skip_serializing_if = "is_default")]
    #[schemars(with = "HashMap::<String, Arg>")]
    pub args: IndexMap<String, Arg>,

    ///
    /// Publicly visible documentation for the field.
    #[serde(default, skip_serializing_if = "is_default")]
    pub doc: Option<String>,

    ///
    /// Allows modifying existing fields.
    #[serde(default, skip_serializing_if = "is_default")]
    pub modify: Option<Modify>,

    ///
    /// Omits a field from public consumption.
    #[serde(default, skip_serializing_if = "is_default")]
    pub omit: Option<Omit>,

    ///
    /// Sets the cache configuration for a field
    pub cache: Option<Cache>,

    ///
    /// Stores the default value for the field
    #[serde(default, skip_serializing_if = "is_default")]
    pub default_value: Option<Value>,

    ///
    /// Marks field as protected by auth provider
    #[serde(default)]
    pub protected: Option<Protected>,

    ///
    /// Resolver for the field
    #[serde(flatten, default, skip_serializing_if = "is_default")]
    pub resolver: Option<Resolver>,
}

// It's a terminal implementation of MergeRight
impl MergeRight for Field {
    fn merge_right(self, other: Self) -> Self {
        other
    }
}

impl Field {
    pub fn has_resolver(&self) -> bool {
        self.resolver.is_some()
    }

    pub fn has_batched_resolver(&self) -> bool {
        self.resolver
            .as_ref()
            .map(Resolver::is_batched)
            .unwrap_or(false)
    }

    pub fn int() -> Self {
        Self { type_of: "Int".to_string().into(), ..Default::default() }
    }

    pub fn string() -> Self {
        Self { type_of: "String".to_string().into(), ..Default::default() }
    }

    pub fn float() -> Self {
        Self { type_of: "Float".to_string().into(), ..Default::default() }
    }

    pub fn boolean() -> Self {
        Self { type_of: "Boolean".to_string().into(), ..Default::default() }
    }

    pub fn id() -> Self {
        Self { type_of: "ID".to_string().into(), ..Default::default() }
    }

    pub fn is_omitted(&self) -> bool {
        self.omit.is_some()
            || self
                .modify
                .as_ref()
                .and_then(|m| m.omit)
                .unwrap_or_default()
    }
}

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    DirectiveDefinition,
    InputDefinition,
)]
#[directive_definition(locations = "FieldDefinition")]
#[serde(deny_unknown_fields)]
pub struct Modify {
    #[serde(default, skip_serializing_if = "is_default")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub omit: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Inline {
    pub path: Vec<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
pub struct Arg {
    #[serde(rename = "type")]
    pub type_of: crate::core::Type,
    #[serde(default, skip_serializing_if = "is_default")]
    pub doc: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub modify: Option<Modify>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub default_value: Option<Value>,
}

#[derive(
    Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema, MergeRight,
)]
pub struct Union {
    pub types: BTreeSet<String>,
    pub doc: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema, MergeRight)]
/// Definition of GraphQL enum type
pub struct Enum {
    pub variants: BTreeSet<Variant>,
    pub doc: Option<String>,
}

/// Definition of GraphQL value
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    schemars::JsonSchema,
    MergeRight,
)]
pub struct Variant {
    pub name: String,
    // directive: alias
    pub alias: Option<Alias>,
}

/// The @alias directive indicates that aliases of one enum value.
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    schemars::JsonSchema,
    MergeRight,
    DirectiveDefinition,
)]
#[directive_definition(locations = "EnumValue")]
pub struct Alias {
    pub options: BTreeSet<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GraphQLOperationType {
    #[default]
    Query,
    Mutation,
}

impl Display for GraphQLOperationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Query => "query",
            Self::Mutation => "mutation",
        })
    }
}

#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema, DirectiveDefinition,
)]
#[directive_definition(repeatable, locations = "Object")]
#[serde(deny_unknown_fields)]
/// The @addField operator simplifies data structures and queries by adding a field that inlines or flattens a nested field or node within your schema. more info [here](https://tailcall.run/docs/guides/operators/#addfield)
pub struct AddField {
    /// Name of the new field to be added
    pub name: String,
    /// Path of the data where the field should point to
    pub path: Vec<String>,
}

impl Config {
    pub fn is_root_operation_type(&self, type_name: &str) -> bool {
        let type_name = type_name.to_lowercase();

        [
            &self.schema.query,
            &self.schema.mutation,
            &self.schema.subscription,
        ]
        .iter()
        .filter_map(|&root_name| root_name.as_ref())
        .any(|root_name| root_name.to_lowercase() == type_name)
    }

    pub fn port(&self) -> u16 {
        self.server.port.unwrap_or(8000)
    }

    pub fn find_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }

    pub fn find_union(&self, name: &str) -> Option<&Union> {
        self.unions.get(name)
    }

    pub fn find_enum(&self, name: &str) -> Option<&Enum> {
        self.enums.get(name)
    }

    pub fn to_yaml(&self) -> Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }

    pub fn to_json(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }

    /// Renders current config to graphQL string
    pub fn to_sdl(&self) -> String {
        crate::core::document::print(self.into())
    }

    pub fn query(mut self, query: &str) -> Self {
        self.schema.query = Some(query.to_string());
        self
    }

    pub fn types(mut self, types: Vec<(&str, Type)>) -> Self {
        let mut graphql_types = BTreeMap::new();
        for (name, type_) in types {
            graphql_types.insert(name.to_string(), type_);
        }
        self.types = graphql_types;
        self
    }

    pub fn contains(&self, name: &str) -> bool {
        self.types.contains_key(name)
            || self.unions.contains_key(name)
            || self.enums.contains_key(name)
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

    pub fn n_plus_one(&self) -> QueryPath {
        super::npo::PathTracker::new(self).find()
    }

    ///
    /// Given a starting type, this function searches for all the unique types
    /// that this type can be connected to via it's fields
    fn find_connections(&self, type_of: &str, mut types: HashSet<String>) -> HashSet<String> {
        if let Some(union_) = self.find_union(type_of) {
            types.insert(type_of.into());

            for type_ in union_.types.iter() {
                types = self.find_connections(type_, types);
            }
        } else if let Some(type_) = self.find_type(type_of) {
            types.insert(type_of.into());
            for (_, field) in type_.fields.iter() {
                if !types.contains(field.type_of.name()) && !self.is_scalar(field.type_of.name()) {
                    types = self.find_connections(field.type_of.name(), types);
                }
            }
        }
        types
    }

    ///
    /// Checks if a type is a scalar or not.
    pub fn is_scalar(&self, type_name: &str) -> bool {
        self.types
            .get(type_name)
            .map_or(Scalar::is_predefined(type_name), |ty| ty.scalar())
    }

    ///
    /// Goes through the complete config and finds all the types that are used
    /// as inputs directly ot indirectly.
    pub fn input_types(&self) -> HashSet<String> {
        self.arguments()
            .iter()
            .filter(|(_, arg)| !self.is_scalar(arg.type_of.name()))
            .map(|(_, arg)| arg.type_of.name())
            .fold(HashSet::new(), |types, type_of| {
                self.find_connections(type_of, types)
            })
    }

    /// finds the all types which are present in union.
    pub fn union_types(&self) -> HashSet<String> {
        self.unions
            .values()
            .flat_map(|union| union.types.iter().cloned())
            .collect()
    }

    /// Returns a list of all the types that are used as output types
    pub fn output_types(&self) -> HashSet<String> {
        let mut types = HashSet::new();

        if let Some(ref query) = &self.schema.query {
            types = self.find_connections(query, types);
        }

        if let Some(ref mutation) = &self.schema.mutation {
            types = self.find_connections(mutation, types);
        }

        types
    }

    /// Returns a list of all the types that are used as interface
    pub fn interface_types(&self) -> HashSet<String> {
        let mut types = HashSet::new();

        for ty in self.types.values() {
            for interface in ty.implements.iter() {
                types.insert(interface.clone());
            }
        }

        types
    }

    /// Returns a list of all the arguments in the configuration
    fn arguments(&self) -> Vec<(&String, &Arg)> {
        self.types
            .iter()
            .flat_map(|(_, type_of)| type_of.fields.iter())
            .flat_map(|(_, field)| field.args.iter())
            .collect::<Vec<_>>()
    }
    /// Removes all types that are passed in the set
    pub fn remove_types(mut self, types: HashSet<String>) -> Self {
        for unused_type in types {
            self.types.remove(&unused_type);
            self.unions.remove(&unused_type);
        }

        self
    }

    pub fn unused_types(&self) -> HashSet<String> {
        let used_types = self.get_all_used_type_names();
        let all_types: HashSet<String> = self
            .types
            .keys()
            .chain(self.unions.keys())
            .cloned()
            .collect();
        all_types.difference(&used_types).cloned().collect()
    }

    /// Gets all the type names used in the schema.
    pub fn get_all_used_type_names(&self) -> HashSet<String> {
        let mut set = HashSet::new();
        let mut stack = Vec::new();
        if let Some(query) = &self.schema.query {
            stack.push(query.clone());
        }
        if let Some(mutation) = &self.schema.mutation {
            stack.push(mutation.clone());
        }
        while let Some(type_name) = stack.pop() {
            if set.contains(&type_name) {
                continue;
            }
            if let Some(union_) = self.unions.get(&type_name) {
                set.insert(type_name);
                for type_ in &union_.types {
                    stack.push(type_.clone());
                }
            } else if let Some(typ) = self.types.get(&type_name) {
                set.insert(type_name);
                for field in typ.fields.values() {
                    stack.extend(field.args.values().map(|arg| arg.type_of.name().to_owned()));
                    stack.push(field.type_of.name().clone());
                }
                for interface in typ.implements.iter() {
                    stack.push(interface.clone())
                }
            }
        }

        set
    }

    pub fn graphql_schema() -> ServiceDocument {
        // Multiple structs may contain a field of the same type when creating directive
        // definitions. To avoid generating the same GraphQL type multiple times,
        // this hash set is used to track visited types and ensure no duplicates are
        // generated.
        let mut generated_types: HashSet<String> = HashSet::new();
        let generated_types = &mut generated_types;

        let builder = ServiceDocumentBuilder::new();
        builder
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
            .add_input(GraphQL::input_definition())
            .add_input(Grpc::input_definition())
            .add_input(Http::input_definition())
            .add_input(Expr::input_definition())
            .add_input(JS::input_definition())
            .add_input(Modify::input_definition())
            .add_input(Cache::input_definition())
            .add_input(Telemetry::input_definition())
            .add_scalar(Scalar::Bytes.scalar_definition())
            .add_scalar(Scalar::Date.scalar_definition())
            .add_scalar(Scalar::Email.scalar_definition())
            .add_scalar(Scalar::Empty.scalar_definition())
            .add_scalar(Scalar::Int128.scalar_definition())
            .add_scalar(Scalar::Int16.scalar_definition())
            .add_scalar(Scalar::Int32.scalar_definition())
            .add_scalar(Scalar::Int64.scalar_definition())
            .add_scalar(Scalar::Int8.scalar_definition())
            .add_scalar(Scalar::JSON.scalar_definition())
            .add_scalar(Scalar::PhoneNumber.scalar_definition())
            .add_scalar(Scalar::UInt128.scalar_definition())
            .add_scalar(Scalar::UInt16.scalar_definition())
            .add_scalar(Scalar::UInt32.scalar_definition())
            .add_scalar(Scalar::UInt64.scalar_definition())
            .add_scalar(Scalar::UInt8.scalar_definition())
            .add_scalar(Scalar::Url.scalar_definition())
            .build()
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_field_has_or_not_batch_resolver() {
        let f1 = Field { ..Default::default() };

        let f2 = Field {
            resolver: Some(Resolver::Http(Http {
                batch_key: vec!["id".to_string()],
                ..Default::default()
            })),
            ..Default::default()
        };

        let f3 = Field {
            resolver: Some(Resolver::Http(Http {
                batch_key: vec![],
                ..Default::default()
            })),
            ..Default::default()
        };

        assert!(!f1.has_batched_resolver());
        assert!(f2.has_batched_resolver());
        assert!(!f3.has_batched_resolver());
    }

    #[test]
    fn test_graphql_directive_name() {
        let name = GraphQL::directive_name();
        assert_eq!(name, "graphQL");
    }

    #[test]
    fn test_from_sdl_empty() {
        let actual = Config::from_sdl("type Foo {a: Int}").to_result().unwrap();
        let expected = Config::default().types(vec![(
            "Foo",
            Type::default().fields(vec![("a", Field::int())]),
        )]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_unused_types_with_cyclic_types() {
        let config = Config::from_sdl(
            "
            type Bar {a: Int}
            type Foo {a: [Foo]}

            type Query {
                foos: [Foo]
            }

            schema {
                query: Query
            }
            ",
        )
        .to_result()
        .unwrap();

        let actual = config.unused_types();
        let mut expected = HashSet::new();
        expected.insert("Bar".to_string());

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_is_root_operation_type_with_query() {
        let mut config = Config::default();
        config.schema.query = Some("Query".to_string());

        assert!(config.is_root_operation_type("Query"));
        assert!(!config.is_root_operation_type("Mutation"));
        assert!(!config.is_root_operation_type("Subscription"));
    }

    #[test]
    fn test_is_root_operation_type_with_mutation() {
        let mut config = Config::default();
        config.schema.mutation = Some("Mutation".to_string());

        assert!(!config.is_root_operation_type("Query"));
        assert!(config.is_root_operation_type("Mutation"));
        assert!(!config.is_root_operation_type("Subscription"));
    }

    #[test]
    fn test_is_root_operation_type_with_subscription() {
        let mut config = Config::default();
        config.schema.subscription = Some("Subscription".to_string());

        assert!(!config.is_root_operation_type("Query"));
        assert!(!config.is_root_operation_type("Mutation"));
        assert!(config.is_root_operation_type("Subscription"));
    }

    #[test]
    fn test_is_root_operation_type_with_no_root_operation() {
        let config = Config::default();

        assert!(!config.is_root_operation_type("Query"));
        assert!(!config.is_root_operation_type("Mutation"));
        assert!(!config.is_root_operation_type("Subscription"));
    }

    #[test]
    fn test_union_types() {
        let sdl = std::fs::read_to_string(tailcall_fixtures::configs::UNION_CONFIG).unwrap();
        let config = Config::from_sdl(&sdl).to_result().unwrap();
        let union_types = config.union_types();
        let expected_union_types: HashSet<String> = ["Bar", "Baz", "Foo"]
            .iter()
            .cloned()
            .map(String::from)
            .collect();
        assert_eq!(union_types, expected_union_types);
    }
}
