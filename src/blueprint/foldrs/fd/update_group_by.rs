use crate::blueprint::FieldDefinition;
use crate::config::{self, Config};
use crate::http::Method;
use crate::lambda::{Expression, Operation};
use crate::try_fold::TryFolding;
use crate::valid::{Valid, ValidExtensions};

pub struct GroupByFold {
  pub field: config::Field,
}

impl TryFolding for GroupByFold {
  type Input = Config;
  type Value = FieldDefinition;
  type Error = String;

  fn try_fold(self, _cfg: &Self::Input, mut field_definition: Self::Value) -> Valid<Self::Value, Self::Error> {
    if let Some(batch) = self.field.group_by.as_ref() {
      if let Some(http) = self.field.http.as_ref() {
        if http.method != Method::GET {
          Valid::fail("GroupBy is only supported for GET requests".to_string())
        } else {
          if let Some(Expression::Unsafe(Operation::Endpoint(request_template, _group_by, dl))) =
            field_definition.resolver
          {
            field_definition.resolver = Some(Expression::Unsafe(Operation::Endpoint(
              request_template.clone(),
              Some(batch.clone()),
              dl,
            )));
          }
          Ok(field_definition)
        }
      } else {
        Valid::fail("GroupBy is only supported for HTTP resolvers".to_string())
      }
    } else {
      Ok(field_definition)
    }
    .trace("@groupBy")
  }
}
