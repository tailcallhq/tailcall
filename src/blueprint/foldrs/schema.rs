use clap::builder::Str;

use crate::blueprint::from_config::validate_field_has_resolver;
use crate::blueprint::transform::Transform;
use crate::blueprint::{Blueprint, SchemaDefinition};
use crate::config::Config;
use crate::try_fold::TryFolding;
use crate::valid::{OptionExtension, Valid, ValidExtensions, VectorExtension};

/// Transform the config blueprint schema
pub struct SchemaFold;

impl From<SchemaFold> for Transform<Config, Blueprint, String> {
  fn from(_value: SchemaFold) -> Self {
    Transform::new(|config: &Config, mut blueprint: Blueprint| {
      let query_type_name = config
        .graphql
        .schema
        .query
        .as_ref()
        .validate_some("Query root is missing".to_owned())?;

      validate_query(config).validate_or(validate_mutation(config))?;

      blueprint.schema = SchemaDefinition {
        query: query_type_name.clone(),
        mutation: config.graphql.schema.mutation.clone(),
        directives: vec![],
      };
      Ok(blueprint)
    })
  }
}

impl TryFolding for SchemaFold {
  type Input = Config;
  type Value = Blueprint;
  type Error = String;

  fn try_fold(self, cfg: &Self::Input, mut blueprint: Self::Value) -> Valid<Self::Value, Self::Error> {
    let query_type_name = cfg
      .graphql
      .schema
      .query
      .as_ref()
      .validate_some("Query root is missing".to_string())?;

    validate_query(cfg).validate_or(validate_mutation(cfg))?;

    blueprint.schema = SchemaDefinition {
      query: query_type_name.clone(),
      mutation: cfg.graphql.schema.mutation.clone(),
      directives: Vec::with_capacity(0), // We'll re-intalze it anyway
    };
    Ok(blueprint)
  }
}

fn validate_query(cfg: &Config) -> Valid<(), String> {
  let query_type_name = cfg
    .graphql
    .schema
    .query
    .as_ref()
    .validate_some("Query root is missing".to_owned())?;

  let Some(query) = cfg.find_type(query_type_name) else {
    return Valid::fail("Query type is not defined".to_owned()).trace(query_type_name);
  };

  query
    .fields
    .iter()
    .validate_all(validate_field_has_resolver)
    .trace(query_type_name)?;

  Ok(())
}

fn validate_mutation(cfg: &Config) -> Valid<(), String> {
  let mutation_type_name = cfg.graphql.schema.mutation.as_ref();

  if let Some(mutation_type_name) = mutation_type_name {
    let Some(mutation) = cfg.find_type(mutation_type_name) else {
      return Valid::fail("Mutation type is not defined".to_owned()).trace(mutation_type_name);
    };

    mutation
      .fields
      .iter()
      .validate_all(validate_field_has_resolver)
      .trace(mutation_type_name)?;
  }

  Ok(())
}
