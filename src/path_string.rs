use std::borrow::Cow;

// use serde_json::json;

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



// best perform 

impl PathString for EvaluationContext<'_> {
    fn path_string(&self, path: &[String]) -> Option<Cow<'_, str>> {
        let (head, tail) = path.split_first()?;
        Some(match head.as_str() {
            "value" => self.path_value(tail)?.to_string().into(),
            "args" => self.args()?.get_path(tail)?.to_string().into(),
            "headers" => self.get_header_as_value(&tail[0])?.to_string().into(),
            "vars" => self.req_ctx.server.vars.clone()?.get(tail[0].as_str())?.to_owned().into(),
            _ => return None,
        })
    }
}




// ---> some optimization and refactored version

// impl PathString for EvaluationContext<'_> {
//     fn path_string(&self, path: &[String]) -> Option<Cow<'_, str>> {
//         let mut result = None;
//         if let Some((head, tail)) = path.split_first() {
//             result = match head.as_str() {
//                 "value" => self.path_value(tail).cloned(),
//                 "args" => self.args()?.get_path(tail).cloned(),
//                 "headers" => self.get_header_as_value(&tail[0]),
//                 "vars" => self
//                     .req_ctx
//                     .server
//                     .vars
//                     .as_ref()?
//                     .get(&tail[0])
//                     .cloned()
//                     .map(async_graphql::Value::String),
//                 _ => None,
//             }
//             .and_then(|v| match v {
//                 async_graphql::Value::String(s) => Some(Cow::Owned(s)),
//                 async_graphql::Value::Number(n) => Some(Cow::Owned(n.to_string())),
//                 async_graphql::Value::Boolean(b) => Some(Cow::Owned(b.to_string())),
//                 async_graphql::Value::Object(map) => Some(json!(map).to_string().into()),
//                 async_graphql::Value::List(list) => Some(json!(list).to_string().into()),
//                 _ => None,
//             });
//         }
//         result
//     }
// }



//  -----> original version
// TODO: improve performance
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
