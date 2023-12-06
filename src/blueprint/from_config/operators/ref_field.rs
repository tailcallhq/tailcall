use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field};
use crate::lambda::Expression;
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn update_ref_field<'a>(
  is_add_field: bool,
) -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(
    move |(config, field, _, _), b_field| {
      if is_add_field {
        return Valid::succeed(b_field);
      }
      if !field.has_resolver() {
        let field_type = config.types.get(&field.type_of);
        if let Some(ty) = field_type {
          let mut mut_field = b_field;
          let has_resolver =
            from_config::schema::validate_field_of_type_has_resolver(&field.type_of, ty, &config.types);
          if has_resolver.is_succeed() {
            mut_field.resolver = Some(Expression::Literal(serde_json::Value::Object(Default::default())));
          }
          Valid::succeed(mut_field)
        } else if is_scalar(&field.type_of) {
          return Valid::succeed(b_field);
        } else {
          return Valid::fail(format!("Unknown type: {}", &field.type_of));
        }
      } else {
        Valid::succeed(b_field)
      }
    },
  )
}
