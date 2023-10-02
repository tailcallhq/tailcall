use std::borrow::Cow;

use serde_json::json;

use crate::json::JsonLike;
use crate::lambda::EvaluationContext;

pub trait PathString {
  fn path_string(&self, path: &[String]) -> Option<Cow<'_, str>>;
}

impl PathString for serde_json::Value {
  fn path_string(&self, path: &[String]) -> Option<Cow<'_, str>> {
    self.get_path(path).and_then(|a| match a {
      serde_json::Value::String(s) => Some(Cow::Borrowed(s.as_str())),
      serde_json::Value::Number(n) => Some(Cow::Owned(n.to_string())),
      serde_json::Value::Bool(b) => Some(Cow::Owned(b.to_string())),
      _ => None,
    })
  }
}

//optimised
impl PathString for EvaluationContext<'_> {
  fn path_string(&self, path: &[String]) -> Option<Cow<'_, str>> {
    if let Some((head, tail)) = path.split_first() {
      let value = match head.as_str() {
        "value" => self.path_value(tail)?.clone(),
        "args" => self.args()?.get_path(tail).cloned()?,
        "headers" => {
          if let Some(header) = tail.get(0) {
            self.get_header_as_value(header)?.clone()
          } else {
            return None;
          }
        }
        "vars" => {
          if let Some(var) = tail.get(0) {
            async_graphql::Value::String(self.req_ctx.server.vars.clone()?.get(var)?.clone())
          } else {
            return None;
          }
        }
        _ => return None,
      };

      return match value {
        async_graphql::Value::String(s) => Some(Cow::Owned(s)),
        async_graphql::Value::Number(n) => Some(Cow::Owned(n.to_string())),
        async_graphql::Value::Boolean(b) => Some(Cow::Owned(b.to_string())),
        async_graphql::Value::Object(map) => Some(json!(map).to_string().into()),
        async_graphql::Value::List(list) => Some(json!(list).to_string().into()),
        _ => None,
      };
    }

    None
  }
}

//  -----> original version
// impl PathString for EvaluationContext<'_> {
//   fn path_string(&self, path: &[String]) -> Option<Cow<'_, str>> {
//     let ctx = self;
//     let mut result = None;
//     if let Some((head, tail)) = path.split_first() {
//       result = match head.as_str() {
//         "value" => ctx.path_value(tail).map(|v| v.to_owned()),
//         "args" => ctx.args()?.get_path(tail).cloned(),
//         "headers" => ctx.get_header_as_value(&tail[0]),
//         "vars" => Some(async_graphql::Value::String(
//           ctx.req_ctx.server.vars.clone()?.get(&tail[0]).cloned()?,
//         )),
//         _ => None,
//       }
//       .and_then(|v| match v {
//         async_graphql::Value::String(s) => Some(Cow::Owned(s)),
//         async_graphql::Value::Number(n) => Some(Cow::Owned(n.to_string())),
//         async_graphql::Value::Boolean(b) => Some(Cow::Owned(b.to_string())),
//         async_graphql::Value::Object(map) => Some(json!(map).to_string().into()),
//         async_graphql::Value::List(list) => Some(json!(list).to_string().into()),
//         _ => None,
//       });
//     }
//     result
//   }
// }
