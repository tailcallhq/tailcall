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
    Named {
        /// Name of the type
        name: String,
        /// Flag to indicate the type is required.
        #[serde(rename = "required", default, skip_serializing_if = "is_default")]
        non_null: bool,
    },
    List {
        /// Type is a list
        #[serde(rename = "list")]
        of_type: Box<WrappingType>,
        /// Flag to indicate the type is required.
        #[serde(rename = "required", default, skip_serializing_if = "is_default")]
        non_null: bool,
    },
}

impl std::fmt::Debug for WrappingType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WrappingType::Named { name, non_null } => {
                if *non_null {
                    write!(f, "{}!", name)
                } else {
                    write!(f, "{}", name)
                }
            }
            WrappingType::List { of_type, non_null } => {
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
        WrappingType::Named { name: "JSON".to_string(), non_null: false }
    }
}

impl WrappingType {
    /// gets the name of the type
    pub fn name(&self) -> &String {
        match self {
            WrappingType::Named { name, .. } => name,
            WrappingType::List { of_type, .. } => of_type.name(),
        }
    }

    /// checks if the type is nullable
    pub fn is_nullable(&self) -> bool {
        !match self {
            WrappingType::Named { non_null, .. } => *non_null,
            WrappingType::List { non_null, .. } => *non_null,
        }
    }
    /// checks if the type is a list
    pub fn is_list(&self) -> bool {
        matches!(self, WrappingType::List { .. })
    }

    /// convert this type into NonNull type
    pub fn into_required(self) -> Self {
        match self {
            WrappingType::Named { name, .. } => Self::Named { name, non_null: true },
            WrappingType::List { of_type, .. } => Self::List { of_type, non_null: true },
        }
    }

    /// convert this into nullable type
    pub fn into_nullable(self) -> Self {
        match self {
            WrappingType::Named { name, .. } => Self::Named { name, non_null: false },
            WrappingType::List { of_type, .. } => Self::List { of_type, non_null: false },
        }
    }

    /// create a nullable list type from this type
    pub fn into_list(self) -> Self {
        WrappingType::List { of_type: Box::new(self), non_null: false }
    }

    /// convert this type from list to non-list for any level of nesting
    pub fn into_single(self) -> Self {
        match self {
            WrappingType::Named { .. } => self,
            WrappingType::List { of_type, .. } => of_type.into_single(),
        }
    }

    /// replace the name of the underlying type
    pub fn with_name(self, name: String) -> Self {
        match self {
            WrappingType::Named { non_null, .. } => WrappingType::Named { name, non_null },
            WrappingType::List { of_type, non_null } => {
                WrappingType::List { of_type: Box::new(of_type.with_name(name)), non_null }
            }
        }
    }
}

impl From<&async_graphql_types::Type> for WrappingType {
    fn from(value: &async_graphql_types::Type) -> Self {
        let non_null = !value.nullable;

        match &value.base {
            async_graphql_types::BaseType::Named(name) => {
                Self::Named { name: name.to_string(), non_null }
            }
            async_graphql_types::BaseType::List(type_) => {
                Self::List { of_type: Box::new(type_.as_ref().into()), non_null }
            }
        }
    }
}

impl From<&WrappingType> for async_graphql_types::Type {
    fn from(value: &WrappingType) -> Self {
        let nullable = value.is_nullable();

        let base = match value {
            WrappingType::Named { name, .. } => {
                async_graphql_types::BaseType::Named(Name::new(name))
            }
            WrappingType::List { of_type, .. } => async_graphql_types::BaseType::List(Box::new(
                async_graphql_types::Type::from(of_type.deref()),
            )),
        };

        async_graphql_types::Type { base, nullable }
    }
}

impl From<&WrappingType> for async_graphql::dynamic::TypeRef {
    fn from(value: &WrappingType) -> Self {
        let nullable = value.is_nullable();

        let base = match value {
            WrappingType::Named { name, .. } => {
                async_graphql::dynamic::TypeRef::Named(name.to_owned().into())
            }
            WrappingType::List { of_type, .. } => async_graphql::dynamic::TypeRef::List(Box::new(
                async_graphql::dynamic::TypeRef::from(of_type.deref()),
            )),
        };

        if nullable {
            base
        } else {
            async_graphql::dynamic::TypeRef::NonNull(Box::new(base))
        }
    }
}

impl From<String> for WrappingType {
    fn from(value: String) -> Self {
        Self::Named { name: value, non_null: false }
    }
}
