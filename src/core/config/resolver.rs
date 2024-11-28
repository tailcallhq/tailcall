use std::ops::Deref;

use async_graphql::parser::types::ConstDirective;
use async_graphql::Positioned;
use serde::{Deserialize, Serialize};
use tailcall_macros::{CustomResolver, MergeRight};
use tailcall_valid::{Valid, Validator};

use super::{Call, EntityResolver, Expr, GraphQL, Grpc, Http, JS};
use crate::core::directive::DirectiveCodec;
use crate::core::merge_right::MergeRight;

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

#[derive(Default, Clone, Debug, PartialEq, Eq, schemars::JsonSchema)]
pub struct ResolverSet(pub Vec<Resolver>);

impl ResolverSet {
    pub fn has_resolver(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn is_batched(&self) -> bool {
        if self.0.is_empty() {
            false
        } else {
            self.0.iter().all(Resolver::is_batched)
        }
    }
}

// Implement custom serializer to provide backward compatibility for JSON/YAML
// formats when converting config to config file. In case the only one resolver
// is defined serialize it as flatten structure instead of `resolvers: []`
// TODO: this is not required in case Tailcall drop defining type schema in
// json/yaml files
impl Serialize for ResolverSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let resolvers = &self.0;

        if resolvers.len() == 1 {
            resolvers.first().unwrap().serialize(serializer)
        } else {
            resolvers.serialize(serializer)
        }
    }
}

// Implement custom deserializer to provide backward compatibility for JSON/YAML
// formats when parsing config files. In case the `resolvers` field is defined
// in config parse it as vec of [Resolver] and otherwise try to parse it as
// single [Resolver] TODO: this is not required in case Tailcall drop defining
// type schema in json/yaml files
impl<'de> Deserialize<'de> for ResolverSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        use serde_json::Value;

        let mut value = Value::deserialize(deserializer)?;

        if let Value::Object(obj) = &mut value {
            if obj.is_empty() {
                return Ok(ResolverSet::default());
            }

            if let Some(value) = obj.remove("resolvers") {
                let resolvers = serde_json::from_value(value).map_err(Error::custom)?;

                return Ok(Self(resolvers));
            }
        }

        let resolver: Resolver = serde_json::from_value(value).map_err(Error::custom)?;

        Ok(ResolverSet::from(resolver))
    }
}

impl From<Resolver> for ResolverSet {
    fn from(value: Resolver) -> Self {
        Self(vec![value])
    }
}

impl Deref for ResolverSet {
    type Target = Vec<Resolver>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MergeRight for ResolverSet {
    fn merge_right(mut self, other: Self) -> Self {
        for resolver in other.0.into_iter() {
            if !self.0.contains(&resolver) {
                self.0.push(resolver);
            }
        }

        self
    }
}
