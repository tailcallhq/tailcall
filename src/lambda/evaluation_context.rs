use std::borrow::Cow;
use std::collections::BTreeMap;
use std::time::Duration;

use async_graphql::{Name, SelectionField, ServerError, Value};
use derive_setters::Setters;
use once_cell::sync::Lazy;
use reqwest::header::HeaderMap;

use super::{EmptyResolverContext, GraphQLOperationContext, ResolverContextLike};
use crate::config::JoinType;
use crate::http::RequestContext;
use crate::lambda::UrlToFieldNameAndTypePairsMap;

// TODO: rename to ResolverContext
#[derive(Clone, Setters)]
#[setters(strip_option)]
pub struct EvaluationContext<'a, Ctx: ResolverContextLike<'a>> {
  pub req_ctx: &'a RequestContext,
  pub graphql_ctx: &'a Ctx,

  // TODO: JS timeout should be read from server settings
  pub timeout: Duration,
}

static REQUEST_CTX: Lazy<RequestContext> = Lazy::new(RequestContext::default);

impl Default for EvaluationContext<'static, EmptyResolverContext> {
  fn default() -> Self {
    Self::new(&REQUEST_CTX, &EmptyResolverContext)
  }
}

impl<'a, Ctx: ResolverContextLike<'a>> EvaluationContext<'a, Ctx> {
  pub fn new(req_ctx: &'a RequestContext, graphql_ctx: &'a Ctx) -> EvaluationContext<'a, Ctx> {
    Self { timeout: Duration::from_millis(5), req_ctx, graphql_ctx }
  }

  pub fn value(&self) -> Option<&Value> {
    self.graphql_ctx.value()
  }

  pub fn arg<T: AsRef<str>>(&self, path: &[T]) -> Option<&'a Value> {
    let arg = self.graphql_ctx.args()?.get(path[0].as_ref());

    get_path_value(arg?, &path[1..])
  }

  pub fn path_value<T: AsRef<str>>(&self, path: &[T]) -> Option<&'a Value> {
    get_path_value(self.graphql_ctx.value()?, path)
  }

  pub fn headers(&self) -> &HeaderMap {
    &self.req_ctx.req_headers
  }

  pub fn header(&self, key: &str) -> Option<&str> {
    let value = self.headers().get(key)?;

    value.to_str().ok()
  }

  pub fn var(&self, key: &str) -> Option<&str> {
    let vars = &self.req_ctx.server.vars;

    vars.get(key).map(|v| v.as_str())
  }

  pub fn add_error(&self, error: ServerError) {
    self.graphql_ctx.add_error(error)
  }
}

impl<'a, Ctx: ResolverContextLike<'a>> GraphQLOperationContext for EvaluationContext<'a, Ctx> {
  fn selection_set(
    &self,
    type_subgraph_fields: Option<BTreeMap<String, (BTreeMap<String, Vec<(String, String)>>, Vec<JoinType>)>>,
    field_type: Option<String>,
    url: String,
    filter_selection_set: bool,
  ) -> Option<String> {
    let selection_set = self.graphql_ctx.field()?.selection_set();

    if filter_selection_set && type_subgraph_fields.is_some() && field_type.is_some() {
      format_selection_set(selection_set, type_subgraph_fields.unwrap(), field_type.unwrap(), url)
    } else {
      format_selection_set_unfiltered(selection_set)
    }
  }
}

fn format_selection_set<'a>(
  selection_set: impl Iterator<Item = SelectionField<'a>>,
  type_subgraph_fields: BTreeMap<String, (UrlToFieldNameAndTypePairsMap, Vec<JoinType>)>,
  field_type: String,
  url: String,
) -> Option<String> {
  let mut set = selection_set
    .filter_map(|selection_field| {
      type_subgraph_fields.get(&field_type).and_then(|(subgraph_fields, _)| {
        subgraph_fields.get(&url).and_then(|fields| {
          fields
            .iter()
            .find(|(name, _)| name == selection_field.name())
            .map(|(_, child_field_type)| {
              format_selection_field(
                selection_field,
                type_subgraph_fields.clone(),
                child_field_type.to_owned(),
                url.clone(),
              )
            })
        })
      })
    })
    .collect::<Vec<_>>();

  if !set.is_empty() {
    let ids = match type_subgraph_fields.get(&field_type) {
      Some((_, join_types)) => join_types
        .iter()
        .filter(|join_type| join_type.base_url.clone().unwrap_or_default() == url)
        .map(|join_type| join_type.key.clone())
        .collect::<Vec<_>>(),
      None => vec!["id".to_string()],
    };
    for id in ids {
      set.extend([id]);
    }
  }

  if set.is_empty() {
    return None;
  }

  Some(format!("{{ {} }}", set.join(" ")))
}

fn format_selection_set_unfiltered<'a>(selection_set: impl Iterator<Item = SelectionField<'a>>) -> Option<String> {
  let set = selection_set.map(format_selection_field_unfiltered).collect::<Vec<_>>();
  if set.is_empty() {
    return None;
  }
  Some(format!("{{ {} }}", set.join(" ")))
}

fn format_selection_field(
  field: SelectionField,
  type_subgraph_fields: BTreeMap<String, (UrlToFieldNameAndTypePairsMap, Vec<JoinType>)>,
  field_type: String,
  url: String,
) -> String {
  let name = field.name();
  let arguments = format_selection_field_arguments(field);
  let selection_set = format_selection_set(field.selection_set(), type_subgraph_fields, field_type, url);

  if let Some(set) = selection_set {
    format!("{}{} {}", name, arguments, set)
  } else {
    format!("{}{}", name, arguments)
  }
}

fn format_selection_field_unfiltered(field: SelectionField) -> String {
  let name = field.name();
  let arguments = format_selection_field_arguments(field);
  let selection_set = format_selection_set_unfiltered(field.selection_set());

  if let Some(set) = selection_set {
    format!("{}{} {}", name, arguments, set)
  } else {
    format!("{}{}", name, arguments)
  }
}

fn format_selection_field_arguments(field: SelectionField) -> Cow<'static, str> {
  let name = field.name();
  let arguments = field
    .arguments()
    .map_err(|error| {
      log::warn!("Failed to resolve arguments for field {name}, due to error: {error}");
      error
    })
    .unwrap_or_default();

  if arguments.is_empty() {
    return Cow::Borrowed("");
  }

  let args = arguments
    .iter()
    .map(|(name, value)| format!("{}: {}", name, value))
    .collect::<Vec<_>>()
    .join(",");

  Cow::Owned(format!("({})", args))
}

pub fn get_path_value<'a, T: AsRef<str>>(input: &'a Value, path: &[T]) -> Option<&'a Value> {
  let mut value = Some(input);
  for name in path {
    match value {
      Some(Value::Object(map)) => {
        value = map.get(&Name::new(name));
      }

      Some(Value::List(list)) => {
        value = list.get(name.as_ref().parse::<usize>().ok()?);
      }
      _ => return None,
    }
  }

  value
}

#[cfg(test)]
mod tests {
  use async_graphql::Value;
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

    let async_value = Value::from_json(json).unwrap();

    let path = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let result = get_path_value(&async_value, &path);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), &Value::String("d".to_string()));
  }

  #[test]
  fn test_path_not_found() {
    let json = json!(
    {
        "a": {
            "b": "c"
        }
    });

    let async_value = Value::from_json(json).unwrap();

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

    let async_value = Value::from_json(json).unwrap();

    let path = vec!["a".to_string(), "0".to_string(), "b".to_string()];
    let result = get_path_value(&async_value, &path);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), &Value::String("c".to_string()));
  }
}
