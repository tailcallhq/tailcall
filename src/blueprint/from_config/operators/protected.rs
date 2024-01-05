use crate::blueprint::FieldDefinition;
use crate::config::{self, Config, Field, Protected};
use crate::directive::DirectiveCodec;
use crate::lambda::Lambda;
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn update_protected<'a>() -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
  TryFold::<(&Config, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
    |(config, field, type_, _), b_field| {
      Valid::succeed(if field.protected || type_.protected {
        if !config.server.auth.is_some() {
          return Valid::fail("@protected operator is used without defining auth on schema's @server".to_owned())
            .trace(Protected::trace_name().as_str());
        }

        b_field.resolver_or_default(Lambda::context().auth_protected(), |r| r.auth_protected())
      } else {
        b_field
      })
    },
  )
}
