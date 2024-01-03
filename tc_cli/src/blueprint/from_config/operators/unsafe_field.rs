use tc_core::blueprint::FieldDefinition;
use tc_core::lambda::Lambda;
use tc_core::valid::Valid;

use crate::config;
use crate::config::{Config, Field};
use crate::try_fold::TryFold;

pub fn update_unsafe<'a>() -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|(_, field, _, _), b_field| {
    let mut updated_b_field = b_field;
    if let Some(op) = &field.unsafe_operation {
      updated_b_field = updated_b_field.resolver_or_default(Lambda::context().to_unsafe_js(op.script.clone()), |r| {
        r.to_unsafe_js(op.script.clone())
      });
    }
    Valid::succeed(updated_b_field)
  })
}
