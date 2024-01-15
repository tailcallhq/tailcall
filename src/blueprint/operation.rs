use std::fmt::Write;

use async_graphql::dynamic::Schema;
use async_graphql::ValidationMode;

use super::into_schema::{create, SchemaModifiers};
use super::Blueprint;
use crate::valid::{Cause, Valid, ValidationError};

#[derive(Debug)]
pub struct OperationQuery {
  query: String,
  file: String,
}

impl OperationQuery {
  pub fn new(query: String, trace: String) -> Self {
    Self { query, file: trace }
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

  pub async fn validate(&self, schema: &Schema) -> Vec<Cause<String>> {
    schema
      .execute(&self.query)
      .await
      .errors
      .iter()
      .map(|e| self.to_cause(e))
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
