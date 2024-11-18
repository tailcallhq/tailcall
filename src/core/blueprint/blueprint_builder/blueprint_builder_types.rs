use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::{self, Display};

use derive_setters::Setters;
use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall_macros::MergeRight;

use crate::core::config::{
    AddField, Alias, Cache, Directive, Discriminate, Modify, Omit, Protected, Resolver,
};
use crate::core::is_default;
use crate::core::merge_right::MergeRight;

///
/// Represents a GraphQL type.
/// A type can be an object, interface, enum or scalar.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, JsonSchema, MergeRight)]
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
    /// Any additional directives
    #[serde(default, skip_serializing_if = "is_default")]
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
    /// Used to overwrite the default discrimination strategy
    pub discriminate: Option<Discriminate>,

    ///
    /// Resolver for the field
    #[serde(flatten, default, skip_serializing_if = "is_default")]
    pub resolver: Option<Resolver>,

    ///
    /// Any additional directives
    #[serde(default, skip_serializing_if = "is_default")]
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
