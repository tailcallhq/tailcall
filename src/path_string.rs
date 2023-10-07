use std::borrow::Cow;

use serde_json::json;

use crate::json::JsonLike;
use crate::lambda::{EvaluationContext, ResolverContextLike};

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

fn convert_value(value: &async_graphql::Value) -> Option<Cow<'_, str>> {
  match value {
    async_graphql::Value::String(s) => Some(Cow::Borrowed(s.as_str())),
    async_graphql::Value::Number(n) => Some(Cow::Owned(n.to_string())),
    async_graphql::Value::Boolean(b) => Some(Cow::Owned(b.to_string())),
    async_graphql::Value::Object(map) => Some(json!(map).to_string().into()),
    async_graphql::Value::List(list) => Some(json!(list).to_string().into()),
    _ => None,
  }
}

impl<'a, Ctx: ResolverContextLike<'a>> PathString for EvaluationContext<'a, Ctx> {
  fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, str>> {
    let ctx = self;

    path.split_first().and_then(|(head, tail)| {
      assert!(!tail.is_empty());

      match head.as_ref() {
        "value" => convert_value(ctx.path_value(tail)?),
        "args" => convert_value(ctx.arg(tail)?),
        "headers" => ctx.header(tail[0].as_ref()).map(|v| v.into()),
        "vars" => ctx.var(tail[0].as_ref()).map(|v| v.into()),
        _ => None,
      }
    })
  }
}
