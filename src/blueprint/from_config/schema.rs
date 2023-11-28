use std::collections::{BTreeMap, HashMap};

use async_graphql::parser::types::ConstDirective;

use crate::blueprint::*;
use crate::config::{Config, Field, Type};
use crate::directive::DirectiveCodec;
use crate::valid::{Valid, ValidationError};

fn validate_query(config: &Config) -> Valid<(), String> {
  Valid::from_option(config.schema.query.clone(), "Query root is missing".to_owned())
    .and_then(|ref query_type_name| {
      let Some(query) = config.find_type(query_type_name) else {
        return Valid::fail("Query type is not defined".to_owned()).trace(query_type_name);
      };

      Valid::from_iter(query.fields.iter(), |i| validate_field_has_resolver(i, &config.types)).trace(query_type_name)
    })
    .unit()
}

// Return is this type has resolver
// Will recursive check all fields
fn validate_field_of_type_has_resolver(name: &str, ty: &Type, types: &BTreeMap<String, Type>) -> Valid<(), String> {
  Valid::from_iter(ty.fields.iter(), |i| validate_field_has_resolver(i, types))
    .trace(name)
    .unit()
}
fn validate_field_has_resolver((name, field): (&String, &Field), types: &BTreeMap<String, Type>) -> Valid<(), String> {
  Valid::<(), String>::fail("No resolver has been found in the schema".to_owned())
    .when(|| {
      if !field.has_resolver() {
        let f_type = &field.type_of;
        if let Some(ty2) = types.get(f_type) {
          let res = validate_field_of_type_has_resolver(f_type, ty2, types);
          return !res.is_succeed();
        } else {
          return true;
        }
      }
      false
    })
    .trace(name)
}

pub fn to_directive(const_directive: ConstDirective) -> Valid<Directive, String> {
  const_directive
    .arguments
    .into_iter()
    .map(|(k, v)| {
      let value = v.node.into_json();
      if let Ok(value) = value {
        return Ok((k.node.to_string(), value));
      }
      Err(value.unwrap_err())
    })
    .collect::<Result<HashMap<String, serde_json::Value>, _>>()
    .map_err(|e| ValidationError::new(e.to_string()))
    .map(|arguments| Directive { name: const_directive.name.node.clone().to_string(), arguments, index: 0 })
    .into()
}

fn validate_mutation(config: &Config) -> Valid<(), String> {
  let mutation_type_name = config.schema.mutation.as_ref();

  if let Some(mutation_type_name) = mutation_type_name {
    let Some(mutation) = config.find_type(mutation_type_name) else {
      return Valid::fail("Mutation type is not defined".to_owned()).trace(mutation_type_name);
    };

    validate_field_of_type_has_resolver(mutation_type_name, mutation, &config.types)
  } else {
    Valid::succeed(())
  }
}

pub fn to_schema<'a>() -> TryFoldConfig<'a, SchemaDefinition> {
  TryFoldConfig::new(|config, _| {
    validate_query(config)
      .and(validate_mutation(config))
      .and(Valid::from_option(
        config.schema.query.as_ref(),
        "Query root is missing".to_owned(),
      ))
      .zip(to_directive(config.server.to_directive()))
      .map(|(query_type_name, directive)| SchemaDefinition {
        query: query_type_name.to_owned(),
        mutation: config.schema.mutation.clone(),
        directives: vec![directive],
      })
  })
}
