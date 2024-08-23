use std::fmt::Formatter;
use std::ops::Deref;

use async_graphql::parser::types as async_graphql_types;
use async_graphql::Name;
use serde::{Deserialize, Serialize};

use crate::core::is_default;

/// Type to represent GraphQL type usage with modifiers
/// [spec](https://spec.graphql.org/October2021/#sec-Wrapping-Types)
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(untagged)]
pub enum WrappingType {
    NamedType {
        name: String,
        #[serde(rename = "required", default, skip_serializing_if = "is_default")]
        non_null: bool,
    },
    ListType {
        #[serde(rename = "list")]
        of_type: Box<WrappingType>,
        #[serde(rename = "required", default, skip_serializing_if = "is_default")]
        non_null: bool,
    },
}

impl std::fmt::Debug for WrappingType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WrappingType::NamedType { name, non_null } => {
                if *non_null {
                    write!(f, "{}!", name)
                } else {
                    write!(f, "{}", name)
                }
            }
            WrappingType::ListType { of_type, non_null } => {
                if *non_null {
                    write!(f, "[{:?}]!", of_type)
                } else {
                    write!(f, "[{:?}]", of_type)
                }
            }
        }
    }
}

impl Default for WrappingType {
    fn default() -> Self {
        WrappingType::NamedType { name: "JSON".to_string(), non_null: false }
    }
}

impl WrappingType {
    /// gets the name of the type
    pub fn name(&self) -> &String {
        match self {
            WrappingType::NamedType { name, .. } => name,
            WrappingType::ListType { of_type, .. } => of_type.name(),
        }
    }

    /// checks if the type is nullable
    pub fn is_nullable(&self) -> bool {
        !match self {
            WrappingType::NamedType { non_null, .. } => *non_null,
            WrappingType::ListType { non_null, .. } => *non_null,
        }
    }
    /// checks if the type is a list
    pub fn is_list(&self) -> bool {
        matches!(self, WrappingType::ListType { .. })
    }

    pub fn into_required(self) -> Self {
        match self {
            WrappingType::NamedType { name, .. } => Self::NamedType { name, non_null: true },
            WrappingType::ListType { of_type, .. } => Self::ListType { of_type, non_null: true },
        }
    }

    pub fn into_nullable(self) -> Self {
        match self {
            WrappingType::NamedType { name, .. } => Self::NamedType { name, non_null: false },
            WrappingType::ListType { of_type, .. } => Self::ListType { of_type, non_null: false },
        }
    }

    pub fn into_list(self) -> Self {
        WrappingType::ListType { of_type: Box::new(self), non_null: false }
    }

    pub fn into_single(self) -> Self {
        match self {
            WrappingType::NamedType { .. } => self,
            WrappingType::ListType { of_type, .. } => of_type.into_single(),
        }
    }

    pub fn with_type(self, name: String) -> Self {
        match self {
            WrappingType::NamedType { non_null, .. } => WrappingType::NamedType { name, non_null },
            WrappingType::ListType { of_type, non_null } => {
                WrappingType::ListType { of_type: Box::new(of_type.with_type(name)), non_null }
            }
        }
    }
}

impl From<&async_graphql_types::Type> for WrappingType {
    fn from(value: &async_graphql_types::Type) -> Self {
        let non_null = !value.nullable;

        match &value.base {
            async_graphql_types::BaseType::Named(name) => {
                Self::NamedType { name: name.to_string(), non_null }
            }
            async_graphql_types::BaseType::List(type_) => {
                Self::ListType { of_type: Box::new(type_.as_ref().into()), non_null }
            }
        }
    }
}

impl From<&WrappingType> for async_graphql_types::Type {
    fn from(value: &WrappingType) -> Self {
        let nullable = value.is_nullable();

        let base = match value {
            WrappingType::NamedType { name, .. } => {
                async_graphql_types::BaseType::Named(Name::new(name))
            }
            WrappingType::ListType { of_type, .. } => async_graphql_types::BaseType::List(
                Box::new(async_graphql_types::Type::from(of_type.deref())),
            ),
        };

        async_graphql_types::Type { base, nullable }
    }
}

impl From<String> for WrappingType {
    fn from(value: String) -> Self {
        Self::NamedType { name: value, non_null: false }
    }
}
