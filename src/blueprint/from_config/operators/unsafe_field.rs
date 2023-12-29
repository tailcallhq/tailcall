use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field};
use crate::lambda::Lambda;
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn update_unsafe<'a>() -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|(_, field, _, _), b_field| {
    let mut updated_b_field = b_field;
    if let Some(op) = &field.unsafe_operation {
      updated_b_field = updated_b_field.resolver_or_default(Lambda::context().to_unsafe_js(op.script.clone()), |r| {
        r.to_unsafe_js(op.script.clone())
      });
      if let Some(cache) = &field.cache  {
        updated_b_field = updated_b_field.resolver_cached(cache.max_age)
      }
    }
    Valid::succeed(updated_b_field)
  })
}
