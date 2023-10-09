use regex::Regex;

use crate::blueprint::from_config::process_path;
use crate::blueprint::transform::Transform;
use crate::blueprint::transformers::Valid;
use crate::blueprint::{FieldDefinition, Type};
use crate::config;
use crate::config::{Config, InlineType};
use crate::lambda::Lambda;
use crate::valid::ValidExtensions;

pub struct UpdateInlineTransform {
  pub field: config::Field,
  pub type_info: config::Type,
}

impl From<UpdateInlineTransform> for Transform<Config, FieldDefinition, String> {
  fn from(value: UpdateInlineTransform) -> Self {
    Transform::new(move |config, field_definition| value.transform(config, field_definition).trace("@inline"))
  }
}

impl UpdateInlineTransform {
  // Main function to update an inline field
  fn transform(self, config: &Config, field_def: FieldDefinition) -> Valid<FieldDefinition> {
    let inlined_path = self.field.inline.as_ref().map(|x| x.path.clone()).unwrap_or_default();
    let handle_invalid_path = |_field_name: &str, _inlined_path: &[String]| -> Valid<Type> {
      Valid::fail("Inline can't be done because provided path doesn't exist".to_string())
    };
    let has_index = inlined_path.iter().any(|s| {
      let re = Regex::new(r"^\d+$").unwrap();
      re.is_match(s)
    });
    if let Some(InlineType { path }) = self.field.clone().inline {
      return match process_path(
        &inlined_path,
        &self.field,
        &self.type_info,
        false,
        config,
        &handle_invalid_path,
      ) {
        Valid::Ok(of_type) => {
          let mut updated_base_field = field_def;
          let resolver = Lambda::context_path(path.clone());
          if has_index {
            updated_base_field.of_type = Type::NamedType { name: of_type.name().to_string(), non_null: false }
          } else {
            updated_base_field.of_type = of_type;
          }

          updated_base_field = updated_base_field.resolver_or_default(resolver, |r| r.to_input_path(path.clone()));
          Valid::Ok(updated_base_field)
        }
        Valid::Err(err) => Valid::Err(err),
      };
    }
    Valid::Ok(field_def)
  }
}
