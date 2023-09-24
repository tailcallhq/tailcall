use std::borrow::Cow;

use serde_json::json;

use crate::json::JsonLike;
use crate::lambda::EvaluationContext;

// TODO: move to it's own file
pub trait PathString {
  fn any_path(&self, path: &[String]) -> Option<Cow<'_, str>>;
}

impl PathString for serde_json::Value {
  fn any_path(&self, path: &[String]) -> Option<Cow<'_, str>> {
    self.get_path(path).and_then(|a| match a {
      serde_json::Value::String(s) => Some(Cow::Borrowed(s.as_str())),
      serde_json::Value::Number(n) => Some(Cow::Owned(n.to_string())),
      serde_json::Value::Bool(b) => Some(Cow::Owned(b.to_string())),
      _ => None,
    })
  }
}

// TODO: improve performance
impl PathString for EvaluationContext<'_> {
  fn any_path(&self, path: &[String]) -> Option<Cow<'_, str>> {
    let ctx = self;
    let resolver_ctx = ctx.context?;
    let value = resolver_ctx.parent_value.as_value()?;
    let mut result = None;
    if let Some((head, tail)) = path.split_first() {
      result = match head.as_str() {
        "value" => value.get_path(tail).cloned(),
        "args" => ctx.args()?.get_path(tail).cloned(),
        "headers" => ctx.get_header_as_value(&tail[0]),
        "vars" => Some(async_graphql::Value::String(
          ctx.req_ctx.server.vars.clone()?.get(&tail[0]).cloned()?,
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
