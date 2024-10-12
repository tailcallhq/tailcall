use async_graphql::parser::types::ConstDirective;
use async_graphql::Positioned;
use serde::{Deserialize, Serialize};
use tailcall_macros::{CustomResolver, MergeRight};

use super::{Call, EntityResolver, Expr, GraphQL, Grpc, Http, JS};
use crate::core::directive::DirectiveCodec;
use tailcall_valid::{Valid, Validator};

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
