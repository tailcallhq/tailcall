use crate::blueprint::FieldDefinition;
use crate::config::{self, Config, Field};
use crate::lambda::Lambda;
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn update_protected<'a>() -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
  TryFold::<(&Config, &Field, &config::Type, &'a str), FieldDefinition, String>::new(|(_, field, type_, _), b_field| {
    Valid::succeed(if field.protected || type_.protected {
      b_field.resolver_or_default(Lambda::context().auth_protected(), |r| r.auth_protected())
    } else {
      b_field
    })
  })
}
