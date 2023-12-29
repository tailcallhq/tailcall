use std::collections::HashMap;

use async_graphql_value::ConstValue;

use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field};
use crate::lambda::Expression::Literal;
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn update_const_field<'a>(
) -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|(config, field, _, _), b_field| {
    let mut updated_b_field = b_field;
    match field.const_field.as_ref() {
      Some(const_field) => {
        let data = const_field.data.to_owned();
        match ConstValue::from_json(data.to_owned()) {
          Ok(gql_value) => {
            let (schema, schema_map) = to_json_schema_for_field(field, config, HashMap::new());
            match schema.validate(&gql_value, &schema_map).to_result() {
              Ok(_) => {
                updated_b_field.resolver = Some(Literal(data));
                Valid::succeed(updated_b_field)
              }
              Err(err) => Valid::from_validation_err(err.transform(&|a| a.to_owned())),
            }
          }
          Err(e) => Valid::fail(format!("invalid JSON: {}", e)),
        }
      }
      None => Valid::succeed(updated_b_field),
    }
  })
}
