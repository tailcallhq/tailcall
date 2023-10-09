use crate::blueprint::from_config::validate_field_has_resolver;
use crate::blueprint::transform::Transform;
use crate::blueprint::transformers::Valid;
use crate::blueprint::{Blueprint, SchemaDefinition};
use crate::config::Config;
use crate::valid::{OptionExtension, ValidExtensions, VectorExtension};

/// Transform the config blueprint schema
pub struct SchemaTransform;

impl From<SchemaTransform> for Transform<Config, Blueprint, String> {
  fn from(_value: SchemaTransform) -> Self {
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

fn validate_query(config: &Config) -> Valid<()> {
  let query_type_name = config
    .graphql
    .schema
    .query
    .as_ref()
    .validate_some("Query root is missing".to_owned())?;

  let Some(query) = config.find_type(query_type_name) else {
    return Valid::fail("Query type is not defined".to_owned()).trace(query_type_name);
  };

  query
    .fields
    .iter()
    .validate_all(validate_field_has_resolver)
    .trace(query_type_name)?;

  Ok(())
}

fn validate_mutation(config: &Config) -> Valid<()> {
  let mutation_type_name = config.graphql.schema.mutation.as_ref();

  if let Some(mutation_type_name) = mutation_type_name {
    let Some(mutation) = config.find_type(mutation_type_name) else {
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
