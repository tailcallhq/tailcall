use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::{self, Display};

use anyhow::{anyhow, Result};
use async_graphql::parser::types::ServiceDocument;
use derive_setters::Setters;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::IntoEnumIterator;
use tailcall_typedefs_common::directive_definition::DirectiveDefinition;
use tailcall_typedefs_common::input_definition::InputDefinition;
use tailcall_typedefs_common::ServiceDocumentBuilder;
use tailcall_valid::{Valid, Validator};

use super::directive::Directive;
use super::from_document::from_document;
use super::{
    AddField, Alias, Cache, Call, Discriminate, Expr, GraphQL, Grpc, Http, Link, Modify, Omit,
    Protected, ResolverSet, Server, Telemetry, Upstream, JS,
};
use crate::core::config::npo::QueryPath;
use crate::core::config::source::Source;
use crate::core::is_default;
use crate::core::macros::MergeRight;
use crate::core::merge_right::MergeRight;
use crate::core::scalar::Scalar;

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
pub struct RuntimeConfig {
    ///
    /// Dictates how the server behaves and helps tune tailcall for all ingress
    /// requests. Features such as request batching, SSL, HTTP2 etc. can be
    /// configured here.
    #[serde(default, skip_serializing_if = "is_default")]
    pub server: Server,

    ///
    /// Dictates how tailcall should handle upstream requests/responses.
    /// Tuning upstream can improve performance and reliability for connections.
    #[serde(default, skip_serializing_if = "is_default")]
    pub upstream: Upstream,

    ///
    /// A list of all links in the schema.
    #[serde(default, skip_serializing_if = "is_default")]
    pub links: Vec<Link>,

    /// Enable [opentelemetry](https://opentelemetry.io) support
    #[serde(default, skip_serializing_if = "is_default")]
    pub telemetry: Telemetry,
}

#[derive(Clone, Debug, Default, Setters, PartialEq, Eq, MergeRight)]
pub struct Config {
    ///
    /// Dictates how the server behaves and helps tune tailcall for all ingress
    /// requests. Features such as request batching, SSL, HTTP2 etc. can be
    /// configured here.
    pub server: Server,

    ///
    /// Dictates how tailcall should handle upstream requests/responses.
    /// Tuning upstream can improve performance and reliability for connections.
    pub upstream: Upstream,

    ///
    /// Specifies the entry points for query and mutation in the generated
    /// GraphQL schema.
    pub schema: RootSchema,

    ///
    /// A map of all the types in the schema.
    #[setters(skip)]
    pub types: BTreeMap<String, Type>,

    ///
    /// A map of all the union types in the schema.
    pub unions: BTreeMap<String, Union>,

    ///
    /// A map of all the enum types in the schema
    pub enums: BTreeMap<String, Enum>,

    ///
    /// A list of all links in the schema.
    pub links: Vec<Link>,

    /// Enable [opentelemetry](https://opentelemetry.io) support
    pub telemetry: Telemetry,
}

///
/// Represents a GraphQL type.
/// A type can be an object, interface, enum or scalar.
#[derive(Clone, Debug, Default, PartialEq, Eq, MergeRight)]
pub struct Type {
    ///
    /// A map of field name and its definition.
    pub fields: BTreeMap<String, Field>,
    ///
    /// Additional fields to be added to the type
    pub added_fields: Vec<AddField>,
    ///
    /// Documentation for the type that is publicly visible.
    pub doc: Option<String>,
    ///
    /// Interfaces that the type implements.
    pub implements: BTreeSet<String>,
    ///
    /// Setting to indicate if the type can be cached.
    pub cache: Option<Cache>,
    ///
    /// Marks field as protected by auth providers
    pub protected: Option<Protected>,
    ///
    /// Apollo federation entity resolver.
    pub resolvers: ResolverSet,
    ///
    /// Any additional directives
    pub directives: Vec<Directive>,
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
    pub fn has_resolver(&self) -> bool {
        self.resolvers.has_resolver()
    }

    pub fn has_batched_resolver(&self) -> bool {
        self.resolvers.is_batched()
    }

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

#[derive(Clone, Debug, Default, Setters, PartialEq, Eq, MergeRight)]
#[setters(strip_option)]
pub struct RootSchema {
    pub query: Option<String>,
    pub mutation: Option<String>,
    pub subscription: Option<String>,
}

///
/// A field definition containing all the metadata information about resolving a
/// field.
#[derive(Clone, Debug, Default, Setters, PartialEq, Eq)]
#[setters(strip_option)]
pub struct Field {
    ///
    /// Refers to the type of the value the field can be resolved to.
    pub type_of: crate::core::Type,

    ///
    /// Map of argument name and its definition.
    pub args: IndexMap<String, Arg>,

    ///
    /// Publicly visible documentation for the field.
    pub doc: Option<String>,

    ///
    /// Allows modifying existing fields.
    pub modify: Option<Modify>,

    ///
    /// Omits a field from public consumption.
    pub omit: Option<Omit>,

    ///
    /// Sets the cache configuration for a field
    pub cache: Option<Cache>,

    ///
    /// Stores the default value for the field
    pub default_value: Option<Value>,

    ///
    /// Marks field as protected by auth provider
    pub protected: Option<Protected>,

    ///
    /// Used to overwrite the default discrimination strategy
    pub discriminate: Option<Discriminate>,

    ///
    /// Resolver for the field
    pub resolvers: ResolverSet,

    ///
    /// Any additional directives
    pub directives: Vec<Directive>,
}

// It's a terminal implementation of MergeRight
impl MergeRight for Field {
    fn merge_right(self, other: Self) -> Self {
        other
    }
}

impl Field {
    pub fn has_resolver(&self) -> bool {
        self.resolvers.has_resolver()
    }

    pub fn has_batched_resolver(&self) -> bool {
        self.resolvers.is_batched()
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inline {
    pub path: Vec<String>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Arg {
    pub type_of: crate::core::Type,
    pub doc: Option<String>,
    pub modify: Option<Modify>,
    pub default_value: Option<Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, MergeRight)]
pub struct Union {
    pub types: BTreeSet<String>,
    pub doc: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, MergeRight)]
/// Definition of GraphQL enum type
pub struct Enum {
    pub variants: BTreeSet<Variant>,
    pub doc: Option<String>,
}

/// Definition of GraphQL value
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, MergeRight)]
pub struct Variant {
    pub name: String,
    // directive: alias
    pub alias: Option<Alias>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
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

impl RuntimeConfig {
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self> {
        Ok(serde_yaml_ng::from_str(yaml)?)
    }

    pub fn from_source(source: Source, config: &str) -> Result<Self> {
        match source {
            Source::Json => RuntimeConfig::from_json(config),
            Source::Yml => RuntimeConfig::from_yaml(config),
            _ => Err(anyhow!("Only the json/yaml runtime configs are supported")),
        }
    }

    pub fn to_yaml(&self) -> Result<String> {
        Ok(serde_yaml_ng::to_string(self)?)
    }

    pub fn to_json(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }
}

impl Config {
    pub fn with_runtime_config(self, runtime_config: RuntimeConfig) -> Self {
        Self {
            server: runtime_config.server,
            upstream: runtime_config.upstream,
            links: runtime_config.links,
            telemetry: runtime_config.telemetry,
            ..self
        }
    }

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

    pub fn from_sdl(sdl: &str) -> Valid<Self, String> {
        let doc = async_graphql::parser::parse_schema(sdl);
        match doc {
            Ok(doc) => from_document(doc),
            Err(e) => Valid::fail(e.to_string()),
        }
    }

    pub fn from_source(source: Source, content: &str) -> Result<Self> {
        match source {
            Source::GraphQL => Ok(Config::from_sdl(content).to_result()?),
            source => Ok(Config::from(RuntimeConfig::from_source(source, content)?)),
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
        } else if self.find_enum(type_of).is_some() {
            types.insert(type_of.into());
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

    pub fn interfaces_types_map(&self) -> BTreeMap<String, BTreeSet<String>> {
        let mut interfaces_types: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        for (type_name, type_definition) in self.types.iter() {
            for implement_name in type_definition.implements.clone() {
                interfaces_types
                    .entry(implement_name)
                    .or_default()
                    .insert(type_name.clone());
            }
        }

        fn recursive_interface_type_merging(
            types_set: &BTreeSet<String>,
            interfaces_types: &BTreeMap<String, BTreeSet<String>>,
            temp_interface_types: &mut BTreeMap<String, BTreeSet<String>>,
        ) -> BTreeSet<String> {
            let mut types_set_local = BTreeSet::new();

            for type_name in types_set.iter() {
                match (
                    interfaces_types.get(type_name),
                    temp_interface_types.get(type_name),
                ) {
                    (Some(types_set_inner), None) => {
                        let types_set_inner = recursive_interface_type_merging(
                            types_set_inner,
                            interfaces_types,
                            temp_interface_types,
                        );
                        temp_interface_types.insert(type_name.to_string(), types_set_inner.clone());
                        types_set_local = types_set_local.merge_right(types_set_inner);
                    }
                    (Some(_), Some(types_set_inner)) => {
                        types_set_local = types_set_local.merge_right(types_set_inner.clone());
                    }
                    _ => {
                        types_set_local.insert(type_name.to_string());
                    }
                }
            }

            types_set_local
        }

        let mut interfaces_types_map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut temp_interface_types: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (interface_name, types_set) in interfaces_types.iter() {
            let types_set = recursive_interface_type_merging(
                types_set,
                &interfaces_types,
                &mut temp_interface_types,
            );
            interfaces_types_map.insert(interface_name.clone(), types_set);
        }

        interfaces_types_map
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
            .add_directive(Modify::directive_definition(generated_types))
            .add_directive(Omit::directive_definition(generated_types))
            .add_directive(Protected::directive_definition(generated_types))
            .add_directive(Discriminate::directive_definition(generated_types))
            .add_input(GraphQL::input_definition())
            .add_input(Grpc::input_definition())
            .add_input(Http::input_definition())
            .add_input(Expr::input_definition())
            .add_input(JS::input_definition())
            .add_input(Modify::input_definition())
            .add_input(Cache::input_definition());

        for scalar in Scalar::iter() {
            builder = builder.add_scalar(scalar.scalar_definition());
        }

        builder.build()
    }
}

impl From<RuntimeConfig> for Config {
    fn from(config: RuntimeConfig) -> Self {
        Self {
            server: config.server,
            upstream: config.upstream,
            links: config.links,
            telemetry: config.telemetry,
            ..Default::default()
        }
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
    use crate::core::config::Resolver;
    use crate::core::directive::DirectiveCodec;

    #[test]
    fn test_field_has_or_not_batch_resolver() {
        let f1 = Field { ..Default::default() };

        let f2 = Field {
            resolvers: Resolver::Http(Http {
                batch_key: vec!["id".to_string()],
                ..Default::default()
            })
            .into(),
            ..Default::default()
        };

        let f3 = Field {
            resolvers: Resolver::Http(Http { batch_key: vec![], ..Default::default() }).into(),
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

    #[test]
    fn test_interfaces_types_map() {
        let sdl = std::fs::read_to_string(tailcall_fixtures::configs::INTERFACE_CONFIG).unwrap();
        let config = Config::from_sdl(&sdl).to_result().unwrap();
        let interfaces_types_map = config.interfaces_types_map();

        let mut expected_union_types = BTreeMap::new();

        {
            let mut set = BTreeSet::new();
            set.insert("E".to_string());
            set.insert("F".to_string());
            expected_union_types.insert("T0".to_string(), set);
        }

        {
            let mut set = BTreeSet::new();
            set.insert("A".to_string());
            set.insert("E".to_string());
            set.insert("B".to_string());
            set.insert("C".to_string());
            set.insert("D".to_string());
            expected_union_types.insert("T1".to_string(), set);
        }

        {
            let mut set = BTreeSet::new();
            set.insert("B".to_string());
            set.insert("E".to_string());
            set.insert("D".to_string());
            expected_union_types.insert("T2".to_string(), set);
        }

        {
            let mut set = BTreeSet::new();
            set.insert("C".to_string());
            set.insert("E".to_string());
            set.insert("D".to_string());
            expected_union_types.insert("T3".to_string(), set);
        }

        {
            let mut set = BTreeSet::new();
            set.insert("D".to_string());
            expected_union_types.insert("T4".to_string(), set);
        }

        assert_eq!(interfaces_types_map, expected_union_types);
    }
}
