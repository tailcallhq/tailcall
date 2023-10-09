use crate::blueprint::transform::Transform;
use crate::blueprint::transformers::Valid;
use crate::blueprint::FieldDefinition;
use crate::config;
use crate::config::Config;
use crate::lambda::Lambda;

pub struct UpdateUnsafeTransform {
  pub field: config::Field,
}

impl From<UpdateUnsafeTransform> for Transform<Config, FieldDefinition, String> {
  fn from(value: UpdateUnsafeTransform) -> Self {
    Transform::new(move |_config, field_definition| value.transform(field_definition))
  }
}

impl UpdateUnsafeTransform {
  fn transform(self, mut field_def: FieldDefinition) -> Valid<FieldDefinition> {
    if let Some(op) = self.field.unsafe_operation {
      field_def = field_def.resolver_or_default(Lambda::context().to_unsafe_js(op.script.clone()), |r| {
        r.to_unsafe_js(op.script.clone())
      });
    }
    Ok(field_def)
  }
}
