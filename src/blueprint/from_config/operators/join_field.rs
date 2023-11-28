use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field};
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn update_join_field<'a>(
) -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &'a str), FieldDefinition, String>::new(|(_, field, _, _), mut b_field| {
    b_field = b_field.join_field(field.join_field.clone());
    Valid::succeed(b_field)
  })
}
