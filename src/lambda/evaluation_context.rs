use std::time::Duration;

use async_graphql::dynamic::ResolverContext;
#[allow(unused_imports)]
use async_graphql::InputType;
use derive_setters::Setters;
use reqwest::header::HeaderMap;

use crate::http::RequestContext;

// TODO: rename to ResolverContext
#[derive(Clone, Setters)]
#[setters(strip_option)]
pub struct EvaluationContext<'a> {
  pub req_ctx: &'a RequestContext,
  pub context: Option<&'a ResolverContext<'a>>,

  // TODO: JS timeout should be read from server settings
  pub timeout: Duration,
}

lazy_static::lazy_static! {
  static ref REQUEST_CTX: RequestContext = RequestContext::default();
}

impl Default for EvaluationContext<'static> {
  fn default() -> Self {
    Self::new(&REQUEST_CTX)
  }
}

impl<'a> EvaluationContext<'a> {
  pub fn new(req_ctx: &'a RequestContext) -> EvaluationContext<'a> {
    Self { context: None, timeout: Duration::from_millis(5), req_ctx }
  }

  pub fn args(&self) -> Option<async_graphql::Value> {
    let ctx = self.context?;

    Some(async_graphql::Value::Object(ctx.args.as_index_map().clone()))
  }

  pub fn path_value(&'a self, path: &'a Vec<String>) -> Option<&'a async_graphql::Value> {
    get_path_value(self.value()?, path)
  }

  pub fn value(&self) -> Option<&'a async_graphql::Value> {
    let ctx = self.context?;
    ctx.parent_value.as_value()
  }

  pub fn headers(&self) -> &HeaderMap {
    &self.req_ctx.req_headers
  }
  pub fn get_header_as_value(&self, key: &str) -> Option<async_graphql::Value> {
    let value = self.headers().get(key)?;
    Some(async_graphql::Value::String(value.to_str().ok()?.to_string()))
  }
}

fn get_path_value<'a>(input: &'a async_graphql::Value, path: &'a Vec<String>) -> Option<&'a async_graphql::Value> {
  let mut value = Some(input);
  for name in path {
    match value {
      Some(async_graphql::Value::Object(map)) => {
        value = map.get(&async_graphql::Name::new(name));
      }

      Some(async_graphql::Value::List(list)) => {
        value = list.get(name.parse::<usize>().ok()?);
      }
      _ => return None,
    }
  }

  value
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use crate::lambda::evaluation_context::get_path_value;

  #[test]
  fn test_path_value() {
    let json = json!(
    {
        "a": {
            "b": {
                "c": "d"
            }
        }
    });

    let async_value = async_graphql::Value::from_json(json).unwrap();

    let path = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let result = get_path_value(&async_value, &path);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), &async_graphql::Value::String("d".to_string()));
  }

  #[test]
  fn test_path_not_found() {
    let json = json!(
    {
        "a": {
            "b": "c"
        }
    });

    let async_value = async_graphql::Value::from_json(json).unwrap();

    let path = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let result = get_path_value(&async_value, &path);
    assert!(result.is_none());
  }

  #[test]
  fn test_numeric_path() {
    let json = json!(
    {
        "a": [{
            "b": "c"
        }]
    });

    let async_value = async_graphql::Value::from_json(json).unwrap();

    let path = vec!["a".to_string(), "0".to_string(), "b".to_string()];
    let result = get_path_value(&async_value, &path);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), &async_graphql::Value::String("c".to_string()));
  }
}
