use async_graphql::dynamic::Schema;
use async_graphql::ValidationMode;
use derive_setters::Setters;

use super::into_schema::{create, SchemaModifiers};
use super::Blueprint;
use crate::valid::{Cause, Valid, ValidationError};

#[derive(Debug, Setters)]
pub struct Operation {
  query: String,
  trace: Option<String>,
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

pub async fn validate_operation(schema: &Schema, query: &str) -> Vec<Cause<String>> {
  schema
    .execute(query)
    .await
    .errors
    .iter()
    .map(|err| {
      Cause::new(err.message.clone()).trace(
        err
          .locations
          .iter()
          .map(|loc| format!("line {} column {}", loc.line, loc.column))
          .collect(),
      )
    })
    .collect()
}

pub async fn validate_operations(blueprint: &Blueprint, operations: Vec<String>) -> Valid<(), String> {
  match validation_schema(blueprint) {
    Ok(schema) => {
      let mut tasks = vec![];
      futures_util::future::join_all(
        operations
          .iter()
          .map(|op| async { (op.as_str(), tokio::fs::read_to_string(op.clone()).await) }),
      )
      .await;

      for op in operations.iter() {
        match tokio::fs::read_to_string(op).await {
          Ok(operation) => {
            let causes = validate_operation(&schema, &operation).await;
            tasks.push((op.to_string(), causes));
          }
          Err(_) => tasks.push((
            op.to_string(),
            vec![Cause::new(format!("Cannot read file operation file {}", op))],
          )),
        }
      }

      Valid::from_iter(tasks.iter(), |(op, causes)| {
        Valid::<(), String>::from_vec_cause(causes.to_vec()).trace(op)
      })
      .unit()
    }
    Err(e) => Valid::from(Err(e)),
  }
}
