use std::borrow::Cow;

use serde_json::json;

use crate::json::JsonLike;
use crate::lambda::{EvaluationContext, GraphqlContext};

pub trait PathString {
  fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, str>>;
}

impl PathString for serde_json::Value {
  fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, str>> {
    self.get_path(path).and_then(|a| match a {
      serde_json::Value::String(s) => Some(Cow::Borrowed(s.as_str())),
      serde_json::Value::Number(n) => Some(Cow::Owned(n.to_string())),
      serde_json::Value::Bool(b) => Some(Cow::Owned(b.to_string())),
      _ => None,
    })
  }
}

// TODO: improve performance
impl<'a, Ctx: GraphqlContext<'a>> PathString for EvaluationContext<'a, Ctx> {
  fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, str>> {
    let ctx = self;
    let mut result = None;
    if let Some((head, tail)) = path.split_first() {
      result = match head.as_ref() {
        "value" => ctx.path_value(tail).map(|v| v.to_owned()),
        "args" => ctx.args()?.get_path(tail).cloned(),
        "headers" => ctx.get_header_as_value(tail[0].as_ref()),
        "vars" => Some(async_graphql::Value::String(
          ctx.req_ctx.server.vars.get(tail[0].as_ref()).cloned()?,
        )),
        _ => None,
      }
      .and_then(|v| match v {
        async_graphql::Value::String(s) => Some(Cow::Owned(s)),
        async_graphql::Value::Number(n) => Some(Cow::Owned(n.to_string())),
        async_graphql::Value::Boolean(b) => Some(Cow::Owned(b.to_string())),
        async_graphql::Value::Object(map) => Some(json!(map).to_string().into()),
        async_graphql::Value::List(list) => Some(json!(list).to_string().into()),
        _ => None,
      });
    }
    result
  }
}
