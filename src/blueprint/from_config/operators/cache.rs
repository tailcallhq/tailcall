use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;

use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field};
use crate::lambda::{Cache, Expression};
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn update_cache(object_name: &str) -> TryFold<'_, (&Config, &Field, &config::Type, &str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|(_config, field, _, _), b_field| {
    let mut updated_b_field = b_field;
    match updated_b_field.resolver.as_ref() {
      Some(source) => {
        if let Some(cache) = &field.cache {
          let mut hasher = DefaultHasher::new();
          object_name.hash(&mut hasher);
          field.name().hash(&mut hasher);
          let cache = Expression::Cache(Cache::new(hasher, cache.max_age, Box::new(source.clone())));
          updated_b_field.resolver = Some(cache);
        }
        Valid::succeed(updated_b_field)
      }
      None => Valid::succeed(updated_b_field),
    }
  })
}
