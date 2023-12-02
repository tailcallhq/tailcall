use indexmap::IndexMap;

use crate::blueprint::from_config::definitions::to_fields;
use crate::blueprint::*;
use crate::config;
use crate::config::{Config, Field};
use crate::lambda::Expression;
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn update_ref_field<'a>() -> TryFold<'a, (&'a Config, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
  TryFold::<(&Config, &Field, &config::Type, &str), FieldDefinition, String>::new(|(config, field, _, _), b_field| {
    if !field.has_resolver() {
      let field_type = config.types.get(&field.type_of);
      if let Some(ty) = field_type {
        let resolvers = to_fields(ty, config).map(|i| {
          i.iter()
            .filter(|fd| fd.resolver.is_some())
            .map(|fd| (fd.name.clone(), fd.resolver.clone()))
            .collect::<IndexMap<String, Option<Expression>>>()
        });
        let mut mut_field = b_field;
        resolvers.map(|i| {
          if !i.is_empty() {
            mut_field.resolver = Some(Expression::Literal(serde_json::Value::Object(Default::default())));
          }
        });
        Valid::succeed(mut_field)
      } else if is_scalar(&field.type_of) {
        return Valid::succeed(b_field);
      } else {
        return Valid::fail(format!("Unknown type: {}", &field.type_of));
      }
    } else {
      Valid::succeed(b_field)
    }
  })
}
