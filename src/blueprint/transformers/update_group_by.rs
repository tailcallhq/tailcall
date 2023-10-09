use crate::blueprint::transform::Transform;
use crate::blueprint::transformers::Valid;
use crate::blueprint::FieldDefinition;
use crate::config::{self, Config};
use crate::http::Method;
use crate::lambda::{Expression, Operation};
use crate::valid::ValidExtensions;

pub struct GroupByTransform {
  pub field: config::Field,
}

impl From<GroupByTransform> for Transform<Config, FieldDefinition, String> {
  fn from(value: GroupByTransform) -> Self {
    Transform::new(move |config, field_definition| value.transform(config, field_definition).trace("@groupBy"))
  }
}

impl GroupByTransform {
  fn transform(self, _config: &Config, mut field_def: FieldDefinition) -> Valid<FieldDefinition> {
    if let Some(batch) = self.field.group_by.as_ref() {
      if let Some(http) = self.field.http.as_ref() {
        if http.method != Method::GET {
          Valid::fail("GroupBy is only supported for GET requests".to_string())
        } else {
          if let Some(Expression::Unsafe(Operation::Endpoint(request_template, _group_by, dl))) = field_def.resolver {
            field_def.resolver = Some(Expression::Unsafe(Operation::Endpoint(
              request_template.clone(),
              Some(batch.clone()),
              dl,
            )));
          }
          Valid::Ok(field_def)
        }
      } else {
        Valid::fail("GroupBy is only supported for HTTP resolvers".to_string())
      }
    } else {
      Valid::Ok(field_def)
    }
  }
}
