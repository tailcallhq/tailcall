use async_graphql::dynamic::Schema;
use async_graphql::ValidationMode;

use super::into_schema::{create, SchemaModifiers};
use super::Blueprint;
use crate::valid::{Cause, Valid, ValidationError};

#[derive(Debug)]
pub struct OperationQuery {
  query: String,
  trace: Option<String>,
}

impl OperationQuery {
  pub fn new(query: String, trace: Option<String>) -> Self {
    Self { query, trace }
  }

  pub async fn validate(&self, schema: &Schema) -> Vec<Cause<String>> {
    schema
      .execute(&self.query)
      .await
      .errors
      .iter()
      .map(|err| {
        let mut trace = if self.trace.is_some() {
          vec![self.trace.as_ref().unwrap().clone()]
        } else {
          vec![]
        };

        trace.extend(
          err
            .locations
            .iter()
            .map(|loc| format!("line {} column {}", loc.line, loc.column)),
        );

        Cause::new(err.message.clone()).trace(trace)
      })
      .collect()
  }
}

pub fn validation_schema(blueprint: &Blueprint) -> Result<Schema, ValidationError<String>> {
  match create(blueprint, Some(SchemaModifiers { no_resolver: true }))
    .validation_mode(ValidationMode::Strict)
    .disable_introspection()
    .finish()
  {
    Ok(schema) => Ok(schema),
    Err(e) => Err(ValidationError::new(e.to_string())),
  }
}

pub async fn validate_operations(blueprint: &Blueprint, operations: Vec<OperationQuery>) -> Valid<(), String> {
  match validation_schema(blueprint) {
    Ok(schema) => Valid::from_iter(
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
    .unit(),
    Err(e) => Valid::from(Err(e)),
  }
}
