use std::fmt::Formatter;
use std::ops::Deref;

use async_graphql::parser::types as async_graphql_types;
use async_graphql::Name;
use serde::{Deserialize, Serialize};

use crate::core::federation::merge::{FederatedMergeIntersection, FederatedMergeUnion};
use crate::core::is_default;
use crate::core::valid::{Valid, Validator};

/// Type to represent GraphQL type usage with modifiers
/// [spec](https://spec.graphql.org/October2021/#sec-Wrapping-Types)
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(untagged)]
pub enum Type {
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
        of_type: Box<Type>,
        /// Flag to indicate the type is required.
        #[serde(rename = "required", default, skip_serializing_if = "is_default")]
        non_null: bool,
    },
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Named { name, non_null } => {
                if *non_null {
                    write!(f, "{}!", name)
                } else {
                    write!(f, "{}", name)
                }
            }
            Type::List { of_type, non_null } => {
                if *non_null {
                    write!(f, "[{:?}]!", of_type)
                } else {
                    write!(f, "[{:?}]", of_type)
                }
            }
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        Type::Named { name: "JSON".to_string(), non_null: false }
    }
}

impl Type {
    /// gets the name of the type
    pub fn name(&self) -> &String {
        match self {
            Type::Named { name, .. } => name,
            Type::List { of_type, .. } => of_type.name(),
        }
    }

    /// checks if the type is nullable
    pub fn is_nullable(&self) -> bool {
        !match self {
            Type::Named { non_null, .. } => *non_null,
            Type::List { non_null, .. } => *non_null,
        }
    }
    /// checks if the type is a list
    pub fn is_list(&self) -> bool {
        matches!(self, Type::List { .. })
    }

    /// convert this type into NonNull type
    pub fn into_required(self) -> Self {
        match self {
            Type::Named { name, .. } => Self::Named { name, non_null: true },
            Type::List { of_type, .. } => Self::List { of_type, non_null: true },
        }
    }

    /// convert this into nullable type
    pub fn into_nullable(self) -> Self {
        match self {
            Type::Named { name, .. } => Self::Named { name, non_null: false },
            Type::List { of_type, .. } => Self::List { of_type, non_null: false },
        }
    }

    /// create a nullable list type from this type
    pub fn into_list(self) -> Self {
        Type::List { of_type: Box::new(self), non_null: false }
    }

    /// convert this type from list to non-list for any level of nesting
    pub fn into_single(self) -> Self {
        match self {
            Type::Named { .. } => self,
            Type::List { of_type, .. } => of_type.into_single(),
        }
    }

    /// replace the name of the underlying type
    pub fn with_name(self, name: String) -> Self {
        match self {
            Type::Named { non_null, .. } => Type::Named { name, non_null },
            Type::List { of_type, non_null } => {
                Type::List { of_type: Box::new(of_type.with_name(name)), non_null }
            }
        }
    }
}

impl From<&async_graphql_types::Type> for Type {
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

impl From<&Type> for async_graphql_types::Type {
    fn from(value: &Type) -> Self {
        let nullable = value.is_nullable();

        let base = match value {
            Type::Named { name, .. } => async_graphql_types::BaseType::Named(Name::new(name)),
            Type::List { of_type, .. } => async_graphql_types::BaseType::List(Box::new(
                async_graphql_types::Type::from(of_type.deref()),
            )),
        };

        async_graphql_types::Type { base, nullable }
    }
}

impl From<&Type> for async_graphql::dynamic::TypeRef {
    fn from(value: &Type) -> Self {
        let nullable = value.is_nullable();

        let base = match value {
            Type::Named { name, .. } => {
                async_graphql::dynamic::TypeRef::Named(name.to_owned().into())
            }
            Type::List { of_type, .. } => async_graphql::dynamic::TypeRef::List(Box::new(
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

impl From<String> for Type {
    fn from(value: String) -> Self {
        Self::Named { name: value, non_null: false }
    }
}

impl Type {
    fn merge(self, other: Self, non_null_merge: fn(bool, bool) -> bool) -> Valid<Self, String> {
        match (self, other) {
            (
                Type::Named { name, non_null },
                Type::Named { name: other_name, non_null: other_non_null },
            ) => {
                if name != other_name {
                    return Valid::fail(format!(
                        "Type mismatch: expected `{}`, got `{}`",
                        &name, other_name
                    ));
                }

                Valid::succeed(Type::Named {
                    name,
                    // non_null only if type is non_null for both sources
                    non_null: non_null_merge(non_null, other_non_null),
                })
            }
            (
                Type::List { of_type, non_null },
                Type::List { of_type: other_of_type, non_null: other_non_null },
            ) => (*of_type)
                .merge(*other_of_type, non_null_merge)
                .map(|of_type| Type::List {
                    of_type: Box::new(of_type),
                    non_null: non_null_merge(non_null, other_non_null),
                }),
            _ => Valid::fail("Type mismatch: expected list, got singular value".to_string()),
        }
    }
}

impl FederatedMergeIntersection for Type {
    /// Executes merge the way that the result type is non_null
    /// if it is specified as non_null in at least one of the definitions.
    /// That's a narrows merge i.e. the result narrows the input definitions
    /// the way it could be handled by both self and other sources
    fn federated_merge_intersection(self, other: Self) -> Valid<Self, String> {
        #[inline]
        fn non_null_merge(non_null: bool, other_non_null: bool) -> bool {
            non_null || other_non_null
        }

        self.merge(other, non_null_merge)
    }
}

impl FederatedMergeUnion for Type {
    /// Executes merge the way that the result type is non_null only
    /// if it is specified as non_null in both sources.
    /// That's a wide merge i.e. the result wides the input definitions
    /// the way it could be handled by both self and other sources
    fn federated_merge_union(self, other: Self) -> Valid<Self, String> {
        #[inline]
        fn non_null_merge(non_null: bool, other_non_null: bool) -> bool {
            non_null && other_non_null
        }

        self.merge(other, non_null_merge)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod federated_merge_wide {
        use super::*;

        #[test]
        fn test_equal() {
            let a = Type::Named { name: "String".to_owned(), non_null: false };
            let b = Type::Named { name: "String".to_owned(), non_null: false };

            assert_eq!(
                a.federated_merge_union(b),
                Valid::succeed(Type::Named { name: "String".to_owned(), non_null: false })
            );

            let a = Type::List {
                of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: false }),
                non_null: true,
            };
            let b = Type::List {
                of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: false }),
                non_null: true,
            };

            assert_eq!(
                a.federated_merge_union(b),
                Valid::succeed(Type::List {
                    of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: false }),
                    non_null: true,
                })
            );
        }

        #[test]
        fn test_different_non_null() {
            let a = Type::Named { name: "String".to_owned(), non_null: false };
            let b = Type::Named { name: "String".to_owned(), non_null: true };

            assert_eq!(
                a.federated_merge_union(b),
                Valid::succeed(Type::Named { name: "String".to_owned(), non_null: false })
            );

            let a = Type::List {
                of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: false }),
                non_null: false,
            };
            let b = Type::List {
                of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: true }),
                non_null: true,
            };

            assert_eq!(
                a.federated_merge_union(b),
                Valid::succeed(Type::List {
                    of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: false }),
                    non_null: false,
                })
            );
        }

        #[test]
        fn test_different_types() {
            let a = Type::Named { name: "String".to_owned(), non_null: false };
            let b = Type::Named { name: "Int".to_owned(), non_null: false };

            assert_eq!(
                a.federated_merge_union(b),
                Valid::fail("Type mismatch: expected `String`, got `Int`".to_owned())
            );

            let a = Type::List {
                of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: false }),
                non_null: true,
            };
            let b = Type::Named { name: "Int".to_owned(), non_null: false };

            assert_eq!(
                a.federated_merge_union(b),
                Valid::fail("Type mismatch: expected list, got singular value".to_owned())
            );
        }
    }

    mod federated_merge_narrow {
        use super::*;

        #[test]
        fn test_equal() {
            let a = Type::Named { name: "String".to_owned(), non_null: false };
            let b = Type::Named { name: "String".to_owned(), non_null: false };

            assert_eq!(
                a.federated_merge_intersection(b),
                Valid::succeed(Type::Named { name: "String".to_owned(), non_null: false })
            );

            let a = Type::List {
                of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: false }),
                non_null: true,
            };
            let b = Type::List {
                of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: false }),
                non_null: true,
            };

            assert_eq!(
                a.federated_merge_intersection(b),
                Valid::succeed(Type::List {
                    of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: false }),
                    non_null: true,
                })
            );
        }

        #[test]
        fn test_different_non_null() {
            let a = Type::Named { name: "String".to_owned(), non_null: false };
            let b = Type::Named { name: "String".to_owned(), non_null: true };

            assert_eq!(
                a.federated_merge_intersection(b),
                Valid::succeed(Type::Named { name: "String".to_owned(), non_null: true })
            );

            let a = Type::List {
                of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: false }),
                non_null: false,
            };
            let b = Type::List {
                of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: true }),
                non_null: true,
            };

            assert_eq!(
                a.federated_merge_intersection(b),
                Valid::succeed(Type::List {
                    of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: true }),
                    non_null: true,
                })
            );
        }

        #[test]
        fn test_different_types() {
            let a = Type::Named { name: "String".to_owned(), non_null: false };
            let b = Type::Named { name: "Int".to_owned(), non_null: false };

            assert_eq!(
                a.federated_merge_intersection(b),
                Valid::fail("Type mismatch: expected `String`, got `Int`".to_owned())
            );

            let a = Type::List {
                of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: false }),
                non_null: true,
            };
            let b = Type::Named { name: "Int".to_owned(), non_null: false };

            assert_eq!(
                a.federated_merge_intersection(b),
                Valid::fail("Type mismatch: expected list, got singular value".to_owned())
            );
        }
    }
}
