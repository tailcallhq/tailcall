use std::sync::Arc;

use anyhow::{anyhow, Error, Result};
use async_graphql::dynamic::Schema;

use crate::async_graphql_hyper::{GraphQLRequest, GraphQLRequestLike};
use crate::blueprint::{Blueprint, SchemaModifiers};
use crate::http::RequestContext;
use crate::valid::{Valid, Validator};

#[derive(Debug)]
pub struct OperationQuery {
    query: GraphQLRequest,
}

impl OperationQuery {
    pub fn new(query: GraphQLRequest, request_context: Arc<RequestContext>) -> Result<Self> {
        let query = query.data(request_context);
        Ok(Self { query })
    }

    async fn validate(self, schema: &Schema) -> Vec<Error> {
        schema
            .execute(self.query.0)
            .await
            .errors
            .iter()
            .map(|v| anyhow!("{}", v.message.clone()))
            .collect()
    }
}

pub async fn validate_operations(
    blueprint: &Blueprint,
    operations: Vec<OperationQuery>,
) -> Valid<(), String> {
    let schema = blueprint.to_schema_with(SchemaModifiers::default().with_no_resolver());
    Valid::from_iter(
        futures_util::future::join_all(operations.into_iter().map(|op| op.validate(&schema)))
            .await
            .iter(),
        |errors| {
            if errors.is_empty() {
                Valid::succeed(())
            } else {
                Valid::fail("Operation validation failed".to_string())
            }
        },
    )
    .unit()
}
