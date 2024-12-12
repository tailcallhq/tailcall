use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

use async_graphql::{ServerError, Value};
use http::header::HeaderMap;

use super::{GraphQLOperationContext, RelatedFields, ResolverContextLike, SelectionField};
use crate::core::document::print_directives;
use crate::core::http::RequestContext;

// TODO: rename to ResolverContext
#[derive(Clone)]
pub struct EvalContext<'a, Ctx: ResolverContextLike> {
    // Context create for each GraphQL Request
    pub request_ctx: &'a RequestContext,

    // Async GraphQL Context
    // Contains current value and arguments
    graphql_ctx: &'a Ctx,

    // Overridden Value for Async GraphQL Context
    graphql_ctx_value: Option<Arc<Value>>,

    // Overridden Arguments for Async GraphQL Context
    graphql_ctx_args: Option<Arc<Value>>,
}

impl<'a, Ctx: ResolverContextLike> EvalContext<'a, Ctx> {
    pub fn with_value(&mut self, value: Value) -> EvalContext<'a, Ctx> {
        let mut ctx = self.clone();
        ctx.graphql_ctx_value = Some(Arc::new(value));
        ctx
    }

    pub fn with_args(&self, args: Value) -> EvalContext<'a, Ctx> {
        let mut ctx = self.clone();
        ctx.graphql_ctx_args = Some(Arc::new(args));
        ctx
    }

    pub fn is_query(&self) -> bool {
        self.graphql_ctx.is_query()
    }

    pub fn new(req_ctx: &'a RequestContext, graphql_ctx: &'a Ctx) -> EvalContext<'a, Ctx> {
        Self {
            request_ctx: req_ctx,
            graphql_ctx,
            graphql_ctx_value: None,
            graphql_ctx_args: None,
        }
    }

    pub fn value(&self) -> Option<&Value> {
        self.graphql_ctx.value()
    }

    pub fn path_arg<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'a, Value>> {
        // TODO: add unit tests for this
        if let Some(args) = self.graphql_ctx_args.as_ref() {
            get_path_value(args.as_ref(), path).map(|a| Cow::Owned(a.clone()))
        } else if path.is_empty() {
            self.graphql_ctx
                .args()
                .map(|a| Cow::Owned(Value::Object(a.clone())))
        } else {
            let arg = self.graphql_ctx.args()?.get(path[0].as_ref())?;
            get_path_value(arg, &path[1..]).map(Cow::Borrowed)
        }
    }

    pub fn path_value<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'a, Value>> {
        // TODO: add unit tests for this
        if let Some(value) = self.graphql_ctx_value.as_ref() {
            get_path_value(value.as_ref(), path).map(|a| Cow::Owned(a.clone()))
        } else {
            get_path_value(self.graphql_ctx.value()?, path).map(Cow::Borrowed)
        }
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.request_ctx.allowed_headers
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        let value = self.headers().get(key)?;

        value.to_str().ok()
    }

    pub fn env_var(&self, key: &str) -> Option<Cow<'_, str>> {
        self.request_ctx.runtime.env.get(key)
    }

    pub fn var(&self, key: &str) -> Option<&str> {
        let vars = &self.request_ctx.server.vars;

        vars.get(key).map(|v| v.as_str())
    }

    pub fn vars(&self) -> &BTreeMap<String, String> {
        &self.request_ctx.server.vars
    }

    pub fn add_error(&self, error: ServerError) {
        self.graphql_ctx.add_error(error)
    }
}

impl<Ctx: ResolverContextLike> GraphQLOperationContext for EvalContext<'_, Ctx> {
    fn directives(&self) -> Option<String> {
        let selection_field = self.graphql_ctx.field()?;
        selection_field
            .directives()
            .as_ref()
            .map(|directives| print_directives(directives.iter()))
    }

    fn selection_set(&self, related_fields: &RelatedFields) -> Option<String> {
        let selection_field = self.graphql_ctx.field()?;
        format_selection_set(selection_field.selection_set(), related_fields)
    }
}

fn format_selection_set<'a>(
    selection_set: impl Iterator<Item = &'a SelectionField>,
    related_fields: &RelatedFields,
) -> Option<String> {
    let set = selection_set
        .filter_map(|field| {
            // add to set only related fields that should be resolved with current resolver
            related_fields.get(field.name()).map(|related_fields| {
                format_selection_field(field, &related_fields.0, &related_fields.1)
            })
        })
        .collect::<Vec<_>>();

    if set.is_empty() {
        return None;
    }

    Some(format!("{{ {} }}", set.join(" ")))
}

fn format_selection_field(
    field: &SelectionField,
    name: &str,
    related_fields: &RelatedFields,
) -> String {
    let arguments = format_selection_field_arguments(field);
    let selection_set = format_selection_set(field.selection_set(), related_fields);

    let mut output = format!("{}{}", name, arguments);

    if let Some(directives) = field.directives() {
        let directives = print_directives(directives.iter());

        if !directives.is_empty() {
            output.push(' ');
            output.push_str(&directives.escape_default().to_string());
        }
    }

    if let Some(selection_set) = selection_set {
        output.push(' ');
        output.push_str(&selection_set);
    }

    output
}

fn format_selection_field_arguments(field: &SelectionField) -> Cow<'static, str> {
    let arguments = field.arguments();

    if arguments.is_empty() {
        return Cow::Borrowed("");
    }

    let args = arguments
        .iter()
        .map(|(name, value)| format!("{}: {}", name, value))
        .collect::<Vec<_>>()
        .join(",");

    Cow::Owned(format!("({})", args.escape_default()))
}

// TODO: this is the same code as src/json/json_like.rs::get_path
pub fn get_path_value<'a, T: AsRef<str>>(input: &'a Value, path: &[T]) -> Option<&'a Value> {
    let mut value = Some(input);
    for name in path {
        match value {
            Some(Value::Object(map)) => {
                value = map.get(name.as_ref());
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

    use crate::core::ir::eval_context::get_path_value;

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
