use std::fmt::Write;

use async_graphql::dynamic::Schema;

use super::{Blueprint, SchemaModifiers};
use crate::async_graphql_hyper::GraphQLRequest;
use crate::valid::{Cause, Valid, Validator};

#[derive(Debug)]
pub struct OperationQuery {
    query: GraphQLRequest,
    file: String,
}

impl OperationQuery {
    pub fn new(query: String, trace: String) -> anyhow::Result<Self> {
        let query = serde_json::from_str(query.as_str())?;
        Ok(Self { query, file: trace })
    }

    async fn validate(self, schema: &Schema) -> Vec<Cause<String>> {
        let file = self.file.clone();
        schema
            .execute(self.query.0)
            .await
            .errors
            .iter()
            .map(|e| to_cause(file.as_str(), e))
            .collect()
    }
}

fn to_cause(file: &str, err: &async_graphql::ServerError) -> Cause<String> {
    let mut trace = Vec::new();

    for loc in err.locations.iter() {
        let mut message = String::new();
        message.write_str(file).unwrap();
        message
            .write_str(format!(":{}:{}", loc.line, loc.column).as_str())
            .unwrap();

        trace.push(message);
    }

    Cause::new(err.message.clone()).trace(trace)
}

pub async fn validate_operations(
    blueprint: &Blueprint,
    operations: Vec<OperationQuery>,
) -> Valid<(), String> {
    let schema = blueprint.to_schema_with(SchemaModifiers::no_resolver());
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
