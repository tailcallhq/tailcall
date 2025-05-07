use std::collections::BTreeMap;

use indexmap::IndexMap;
use tailcall_valid::{Valid, Validator};

use super::{Cache, ConfigModule};
use crate::core::config::{Arg, Config, Enum, Field, RootSchema, Type};
use crate::core::merge_right::MergeRight;
use crate::core::variance::{Contravariant, Covariant, Invariant};
use crate::core::{self};

impl core::Type {
    fn merge(self, other: Self, non_null_merge: fn(bool, bool) -> bool) -> Valid<Self, String> {
        use core::Type;

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

impl Contravariant for core::Type {
    /// Executes merge the way that the result type is non_null
    /// if it is specified as non_null in at least one of the definitions.
    /// That's a narrows merge i.e. the result narrows the input definitions
    /// the way it could be handled by both self and other sources
    fn shrink(self, other: Self) -> Valid<Self, String> {
        #[inline]
        fn non_null_merge(non_null: bool, other_non_null: bool) -> bool {
            non_null || other_non_null
        }

        self.merge(other, non_null_merge)
    }
}

impl Covariant for core::Type {
    /// Executes merge the way that the result type is non_null only
    /// if it is specified as non_null in both sources.
    /// That's a wide merge i.e. the result wides the input definitions
    /// the way it could be handled by both self and other sources
    fn expand(self, other: Self) -> Valid<Self, String> {
        #[inline]
        fn non_null_merge(non_null: bool, other_non_null: bool) -> bool {
            non_null && other_non_null
        }

        self.merge(other, non_null_merge)
    }
}

impl Contravariant for Arg {
    fn shrink(self, other: Self) -> Valid<Self, String> {
        self.type_of.shrink(other.type_of).map(|type_of| Self {
            type_of,
            doc: self.doc.merge_right(other.doc),
            modify: self.modify.merge_right(other.modify),
            default_value: self.default_value.or(other.default_value),
        })
    }
}

impl Contravariant for Field {
    fn shrink(self, other: Self) -> Valid<Self, String> {
        self.type_of
            .shrink(other.type_of)
            .fuse(self.args.shrink(other.args))
            .map(|(type_of, args)| Self {
                type_of,
                args,
                doc: self.doc.merge_right(other.doc),
                modify: self.modify.merge_right(other.modify),
                omit: self.omit.merge_right(other.omit),
                cache: self.cache.merge_right(other.cache),
                default_value: self.default_value.or(other.default_value),
                protected: self.protected.merge_right(other.protected),
                discriminate: self.discriminate.merge_right(other.discriminate),
                resolvers: self.resolvers.merge_right(other.resolvers),
                directives: self.directives.merge_right(other.directives),
            })
    }
}

impl Covariant for Field {
    fn expand(self, other: Self) -> Valid<Self, String> {
        self.type_of
            .expand(other.type_of)
            // args are always merged with narrow
            .fuse(self.args.shrink(other.args))
            .map(|(type_of, args)| Self {
                type_of,
                args,
                doc: self.doc.merge_right(other.doc),
                modify: self.modify.merge_right(other.modify),
                omit: self.omit.merge_right(other.omit),
                cache: self.cache.merge_right(other.cache),
                default_value: self.default_value.or(other.default_value),
                protected: self.protected.merge_right(other.protected),
                discriminate: self.discriminate.merge_right(other.discriminate),
                resolvers: self.resolvers.merge_right(other.resolvers),
                directives: self.directives.merge_right(other.directives),
            })
    }
}

impl Contravariant for Type {
    fn shrink(self, other: Self) -> Valid<Self, String> {
        self.fields.shrink(other.fields).map(|fields| Self {
            fields,
            // TODO: is not very clear how to merge added_fields here
            added_fields: self.added_fields.merge_right(other.added_fields),
            doc: self.doc.merge_right(other.doc),
            implements: self.implements.merge_right(other.implements),
            cache: self.cache.merge_right(other.cache),
            protected: self.protected.merge_right(other.protected),
            resolvers: self.resolvers.merge_right(other.resolvers),
            directives: self.directives.merge_right(other.directives),
        })
    }
}

impl Covariant for Type {
    fn expand(self, other: Self) -> Valid<Self, String> {
        self.fields.expand(other.fields).map(|fields| Self {
            fields,
            // TODO: is not very clear how to merge added_fields here
            added_fields: self.added_fields.merge_right(other.added_fields),
            doc: self.doc.merge_right(other.doc),
            implements: self.implements.merge_right(other.implements),
            cache: self.cache.merge_right(other.cache),
            protected: self.protected.merge_right(other.protected),
            resolvers: self.resolvers.merge_right(other.resolvers),
            directives: self.directives.merge_right(other.directives),
        })
    }
}

impl Contravariant for Enum {
    fn shrink(mut self, other: Self) -> Valid<Self, String> {
        self.variants.retain(|key| other.variants.contains(key));

        Valid::succeed(Self {
            variants: self.variants,
            doc: self.doc.merge_right(other.doc),
        })
    }
}

impl Covariant for Enum {
    fn expand(mut self, other: Self) -> Valid<Self, String> {
        self.variants.extend(other.variants);

        Valid::succeed(Self {
            variants: self.variants,
            doc: self.doc.merge_right(other.doc),
        })
    }
}

impl Invariant for RootSchema {
    fn unify(self, other: Self) -> Valid<Self, String> {
        fn unify_option<T>(left: Option<T>, right: Option<T>) -> Option<T> {
            match (left, right) {
                (None, None) => None,
                (None, Some(that)) => Some(that),
                (Some(this), _) => Some(this),
            }
        }

        Valid::succeed(Self {
            query: unify_option(self.query, other.query),
            mutation: unify_option(self.mutation, other.mutation),
            subscription: unify_option(self.subscription, other.subscription),
        })
    }
}

impl Invariant for Cache {
    fn unify(self, other: Self) -> Valid<Self, String> {
        let mut types = self.config.types;
        let mut enums = self.config.enums;

        Valid::from_iter(other.config.types, |(type_name, other_type)| {
            let trace_name = type_name.clone();
            match types.remove(&type_name) {
                Some(ty) => {
                    let is_self_input = self.input_types.contains(&type_name);
                    let is_other_input = other.input_types.contains(&type_name);
                    let is_self_output = self.output_types.contains(&type_name)
                        || self.interfaces_types_map.contains_key(&type_name);
                    let is_other_output = other.output_types.contains(&type_name)
                        || other.interfaces_types_map.contains_key(&type_name);

                    match (
                        is_self_input,
                        is_self_output,
                        is_other_input,
                        is_other_output,
                    ) {
                        // both input types
                        (true, false, true, false) => ty.shrink(other_type),
                        // both output types
                        (false, true, false, true) => ty.expand(other_type),
                        // if type is unknown on one side, we merge based on info from another side
                        (false, false, true, false) | (true, false, false, false) => {
                            ty.shrink(other_type)
                        }
                        (false, false, false, true) | (false, true, false, false) => {
                            ty.expand(other_type)
                        }
                        // if type is used as both input and output on either side
                        // generated validation error because we need to merge it differently
                        (true, true, _, _) | (_, _, true, true) => Valid::fail("Type is used both as input and output type that couldn't be merged for federation".to_string()),
                        // type is used differently on both sides
                        (true, false, false, true) | (false, true, true, false) => Valid::fail("Type is used as input type in one subgraph and output type in another".to_string()),
                        (false, false, false, false) => Valid::fail("Cannot infer the usage of type and therefore merge it from the subgraph".to_string()),
                    }
                }
                None => Valid::succeed(other_type),
            }
            .map(|ty| (type_name, ty))
            .trace(&trace_name)
        })
        .fuse(Valid::from_iter(other.config.enums, |(name, other_enum)| {
            let trace_name = name.clone();

            match enums.remove(&name) {
                Some(en) => {
                    let is_self_input = self.input_types.contains(&name);
                    let is_other_input = other.input_types.contains(&name);
                    let is_self_output = self.output_types.contains(&name);
                    let is_other_output = other.output_types.contains(&name);

                    match (is_self_input, is_self_output, is_other_input, is_other_output) {
                        // both input types
                        (true, false, true, false) => en.shrink(other_enum),
                        // both output types
                        (false, true, false, true) => en.expand(other_enum),
                        // if type is unknown on one side, we merge based on info from another side
                        (false, false, true, false) | (true, false, false, false) => {
                            en.shrink(other_enum)
                        }
                        (false, false, false, true) | (false, true, false, false) => {
                            en.expand(other_enum)
                        }
                        // if type is used as both input and output on either side
                        // generated validation error because we need to merge it differently
                        (true, true, _, _) | (_, _, true, true) => {
                            if en == other_enum {
                                Valid::succeed(en)
                            } else {
                                Valid::fail("Enum is used both as input and output types and in that case the enum content should be equal for every subgraph".to_string())
                            }
                        },
                        // type is used differently on both sides
                        (true, false, false, true) | (false, true, true, false) => Valid::fail("Enum is used as input type in one subgraph and output type in another".to_string()),
                        (false, false, false, false) => Valid::fail("Cannot infer the usage of enum and therefore merge it from the subgraph".to_string()),
                    }
                },
                None => Valid::succeed(other_enum),
            }
            .map(|en| (name, en))
            .trace(&trace_name)
        }))
        .fuse(self.config.schema.unify(other.config.schema))
        .map(|(merged_types, merged_enums, schema)| {
            types.extend(merged_types);
            enums.extend(merged_enums);

            let config = Config {
                types,
                enums,
                unions: self.config.unions.merge_right(other.config.unions),
                schema,
                upstream: self.config.upstream.merge_right(other.config.upstream),
                ..self.config
            };

            Cache {
                config,
                input_types: self.input_types.merge_right(other.input_types),
                output_types: self.output_types.merge_right(other.output_types),
                interfaces_types_map: self.interfaces_types_map.merge_right(other.interfaces_types_map),
            }
        })
    }
}

impl Invariant for ConfigModule {
    fn unify(self, other: Self) -> Valid<Self, String> {
        self.cache.unify(other.cache).map(|cache| Self {
            cache,
            extensions: self.extensions.merge_right(other.extensions),
        })
    }
}

trait TypedEntry {
    fn type_of(&self) -> &crate::core::Type;
}

impl TypedEntry for Field {
    fn type_of(&self) -> &crate::core::Type {
        &self.type_of
    }
}

impl TypedEntry for Arg {
    fn type_of(&self) -> &crate::core::Type {
        &self.type_of
    }
}

trait FederatedMergeCollection:
    IntoIterator<Item = (String, Self::Entry)>
    + FromIterator<(String, Self::Entry)>
    + Extend<(String, Self::Entry)>
{
    type Entry: TypedEntry;

    fn remove(&mut self, name: &str) -> Option<Self::Entry>;
}

impl<Entry: TypedEntry> FederatedMergeCollection for IndexMap<String, Entry> {
    type Entry = Entry;

    fn remove(&mut self, name: &str) -> Option<Self::Entry> {
        self.swap_remove(name)
    }
}

impl<Entry: TypedEntry> FederatedMergeCollection for BTreeMap<String, Entry> {
    type Entry = Entry;

    fn remove(&mut self, name: &str) -> Option<Self::Entry> {
        self.remove(name)
    }
}

impl<C> Contravariant for C
where
    C: FederatedMergeCollection,
    C::Entry: Contravariant,
{
    fn shrink(mut self, other: Self) -> Valid<Self, String> {
        Valid::from_iter(other, |(name, other_field)| {
        match self.remove(&name) {
            Some(field) => Contravariant::shrink(field, other_field).map(|merged| Some((name.clone(), merged))),
            None => {
                if other_field.type_of().is_nullable() {
                    Valid::succeed(None)
                } else {
                    Valid::fail("Input arg is marked as non_null on the right side, but is not present on the left side".to_string())
                }
            },
        }
        .trace(&name)
        })
        .fuse(Valid::from_iter(self, |(name, field)| {
            if field.type_of().is_nullable() {
                Valid::succeed(())
            } else {
                Valid::fail("Input arg is marked as non_null on the left side, but is not present on the right side".to_string()).trace(&name)
            }
        }))
        .map(|(merged_fields, _)| {
            merged_fields.into_iter().flatten().collect()
        })
    }
}

impl<C> Covariant for C
where
    C: FederatedMergeCollection,
    C::Entry: Covariant,
{
    fn expand(mut self, other: Self) -> Valid<Self, String> {
        Valid::from_iter(other, |(name, other_field)| match self.remove(&name) {
            Some(field) => field
                .expand(other_field)
                .map(|merged| (name.clone(), merged))
                .trace(&name),
            None => Valid::succeed((name, other_field)),
        })
        .map(|merged_fields| {
            let mut merged_fields: C = merged_fields.into_iter().collect();
            merged_fields.extend(self);

            merged_fields
        })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use insta::assert_snapshot;
    use tailcall_valid::Validator;

    use super::*;
    use crate::core::config::ConfigModule;
    use crate::include_config;

    #[test]
    fn test_types_valid() -> Result<()> {
        let types1 = ConfigModule::from(include_config!("./fixtures/types-1.graphql")?);
        let types2 = ConfigModule::from(include_config!("./fixtures/types-2.graphql")?);

        let merged = types1.unify(types2).to_result()?;

        assert_snapshot!(merged.to_sdl());

        Ok(())
    }

    #[test]
    fn test_types_invalid() -> Result<()> {
        let types1 = ConfigModule::from(include_config!("./fixtures/types-1.graphql")?);
        let types3 = ConfigModule::from(include_config!("./fixtures/types-3.graphql")?);

        let merged = types1.unify(types3).to_result();

        assert_snapshot!(merged.unwrap_err());

        Ok(())
    }

    #[test]
    fn test_unions_valid() -> Result<()> {
        let unions1 = ConfigModule::from(include_config!("./fixtures/unions-1.graphql")?);
        let unions2 = ConfigModule::from(include_config!("./fixtures/unions-2.graphql")?);

        let merged = unions1.unify(unions2).to_result()?;

        assert_snapshot!(merged.to_sdl());

        Ok(())
    }

    #[test]
    fn test_enums_valid() -> Result<()> {
        let enums1 = ConfigModule::from(include_config!("./fixtures/enums-1.graphql")?);
        let enums2 = ConfigModule::from(include_config!("./fixtures/enums-2.graphql")?);

        let merged = enums1.unify(enums2).to_result()?;

        assert_snapshot!(merged.to_sdl());

        Ok(())
    }

    #[test]
    fn test_enums_invalid() -> Result<()> {
        let enums1 = ConfigModule::from(include_config!("./fixtures/enums-1.graphql")?);
        let enums3 = ConfigModule::from(include_config!("./fixtures/enums-3.graphql")?);

        let merged = enums1.unify(enums3).to_result();

        assert_snapshot!(merged.unwrap_err());

        Ok(())
    }

    #[test]
    fn test_federation_router() -> Result<()> {
        let router = ConfigModule::from(include_config!("./fixtures/router.graphql")?);

        let subgraph_users =
            ConfigModule::from(include_config!("./fixtures/subgraph-users.graphql")?);

        let subgraph_posts =
            ConfigModule::from(include_config!("./fixtures/subgraph-posts.graphql")?);

        let merged = router;
        let merged = merged.unify(subgraph_users).to_result()?;
        let merged = merged.unify(subgraph_posts).to_result()?;

        assert_snapshot!(merged.to_sdl());

        Ok(())
    }

    #[test]
    fn test_upstream_allowed_headers_propagation() -> Result<()> {
        use std::collections::BTreeSet;

        use crate::core::config::Config;

        // Create a Config with no allowed_headers
        let config1 = Config::default();

        // Create a Config with allowed_headers
        let mut config2 = Config::default();
        let mut headers = BTreeSet::new();
        headers.insert("x-user-id".to_string());
        headers.insert("authorization".to_string());
        config2.upstream.allowed_headers = Some(headers.clone());

        // Create Cache instances
        let cache1 = Cache::from(config1.clone());
        let cache2 = Cache::from(config2.clone());

        // Verify initial state
        assert_eq!(cache1.config.upstream.allowed_headers, None);
        assert_eq!(
            cache2.config.upstream.allowed_headers,
            Some(headers.clone())
        );

        // Test merging cache1 and cache2
        let merged = cache1.clone().unify(cache2.clone()).to_result()?;

        // Verify that allowed_headers from cache2 are preserved in the merged cache
        assert_eq!(
            merged.config.upstream.allowed_headers,
            Some(headers.clone())
        );

        // Test the reverse order (cache2 and cache1)
        let merged_reverse = cache2.unify(cache1).to_result()?;

        // Verify that allowed_headers from cache2 are still preserved
        assert_eq!(
            merged_reverse.config.upstream.allowed_headers,
            Some(headers)
        );

        Ok(())
    }

    #[test]
    fn test_upstream_merge_right_order() -> Result<()> {
        use crate::core::config::Config;

        // Create a Config with specific upstream settings
        let mut config1 = Config::default();
        config1.upstream.connect_timeout = Some(30);
        config1.upstream.timeout = Some(60);

        // Create another Config with different upstream settings
        let mut config2 = Config::default();
        config2.upstream.timeout = Some(120); // This should override config1's timeout
        config2.upstream.http_cache = Some(1000); // This should be added to the merged config

        // Create Cache instances
        let cache1 = Cache::from(config1.clone());
        let cache2 = Cache::from(config2.clone());

        // Test merging cache1 and cache2
        let merged = cache1.clone().unify(cache2.clone()).to_result()?;

        // Verify that values from cache2 override those from cache1
        assert_eq!(merged.config.upstream.connect_timeout, Some(30));
        assert_eq!(merged.config.upstream.timeout, Some(120)); // Should be from config2
        assert_eq!(merged.config.upstream.http_cache, Some(1000)); // Should be from config2

        // Test the reverse order (cache2 and cache1)
        let merged_reverse = cache2.unify(cache1).to_result()?;

        // Verify that values from cache1 override those from cache2
        assert_eq!(merged_reverse.config.upstream.connect_timeout, Some(30)); // Should be from config1
        assert_eq!(merged_reverse.config.upstream.timeout, Some(60)); // Should be from config1
        assert_eq!(merged_reverse.config.upstream.http_cache, Some(1000)); // Should be from config2

        Ok(())
    }

    mod core_type {
        use super::*;
        use crate::core::Type;

        mod expand {
            use super::*;

            #[test]
            fn test_equal() {
                let a = Type::Named { name: "String".to_owned(), non_null: false };
                let b = Type::Named { name: "String".to_owned(), non_null: false };

                assert_eq!(
                    a.expand(b),
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
                    a.expand(b),
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
                    a.expand(b),
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
                    a.expand(b),
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
                    a.expand(b),
                    Valid::fail("Type mismatch: expected `String`, got `Int`".to_owned())
                );

                let a = Type::List {
                    of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: false }),
                    non_null: true,
                };
                let b = Type::Named { name: "Int".to_owned(), non_null: false };

                assert_eq!(
                    a.expand(b),
                    Valid::fail("Type mismatch: expected list, got singular value".to_owned())
                );
            }
        }

        mod shrink {
            use super::*;

            #[test]
            fn test_equal() {
                let a = Type::Named { name: "String".to_owned(), non_null: false };
                let b = Type::Named { name: "String".to_owned(), non_null: false };

                assert_eq!(
                    a.shrink(b),
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
                    a.shrink(b),
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
                    a.shrink(b),
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
                    a.shrink(b),
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
                    a.shrink(b),
                    Valid::fail("Type mismatch: expected `String`, got `Int`".to_owned())
                );

                let a = Type::List {
                    of_type: Box::new(Type::Named { name: "Int".to_owned(), non_null: false }),
                    non_null: true,
                };
                let b = Type::Named { name: "Int".to_owned(), non_null: false };

                assert_eq!(
                    a.shrink(b),
                    Valid::fail("Type mismatch: expected list, got singular value".to_owned())
                );
            }
        }
    }
}
