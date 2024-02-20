use std::fmt::Write;

use async_graphql::dynamic::Schema;
use async_graphql_value::ConstValue;

use super::{Blueprint, SchemaModifiers};
use crate::valid::{Cause, Valid, Validator};

#[derive(Debug)]
pub struct OperationQuery {
    query: String,
    file: String,
    variables: async_graphql::Variables,
}

impl OperationQuery {
    pub fn new(query: String, trace: String) -> Self {
        Self {
            query,
            file: trace,
            variables: async_graphql::Variables::from_value(ConstValue::Null),
        }
    }

    pub fn new_with_variables(
        query: String,
        trace: String,
        variables: async_graphql::Variables,
    ) -> Self {
        Self { query, file: trace, variables }
    }

    fn to_cause(&self, err: &async_graphql::ServerError) -> Cause<String> {
        let mut trace = Vec::new();
        let file = self.file.as_str();

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

    async fn validate(&self, schema: &Schema) -> Vec<Cause<String>> {
        let request: async_graphql::Request = self.query.as_str().into();
        let request = request.variables(self.variables.clone());
        schema
            .execute(request)
            .await
            .errors
            .iter()
            .map(|e| self.to_cause(e))
            .collect()
    }
}

pub async fn validate_operations(
    blueprint: &Blueprint,
    operations: Vec<OperationQuery>,
) -> Valid<(), String> {
    let schema = blueprint.to_schema_with(SchemaModifiers::no_resolver());
    Valid::from_iter(
        futures_util::future::join_all(operations.iter().map(|op| op.validate(&schema)))
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
