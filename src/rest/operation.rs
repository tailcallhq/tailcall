use std::fmt::Write;
use std::sync::Arc;

use async_graphql::dynamic::Schema;

use crate::async_graphql_hyper::{GraphQLRequest, GraphQLRequestLike};
use crate::blueprint::{Blueprint, SchemaModifiers};
use crate::http::RequestContext;
use crate::valid::{Cause, Valid, Validator};

#[derive(Debug)]
pub struct OperationQuery {
    query: GraphQLRequest,
}

impl OperationQuery {
    pub fn new(
        query: GraphQLRequest,
        request_context: Arc<RequestContext>,
    ) -> anyhow::Result<Self> {
        let query = query.data(request_context);
        Ok(Self { query })
    }

    async fn validate(self, schema: &Schema) -> Vec<Cause<String>> {
        schema
            .execute(self.query.0)
            .await
            .errors
            .iter()
            .map(to_cause)
            .collect()
    }
}

fn to_cause(err: &async_graphql::ServerError) -> Cause<String> {
    let mut trace = Vec::new();

    for loc in err.locations.iter() {
        let mut message = String::new();
        message
            .write_str(format!("{}:{}", loc.line, loc.column).as_str())
            .unwrap();

        trace.push(message);
    }

    Cause::new(err.message.clone()).trace(trace)
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
                Valid::<(), String>::from_vec_cause(errors.to_vec())
            }
        },
    )
    .unit()
}
