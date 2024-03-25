use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use async_graphql::{SelectionField, ServerError, Value};
use reqwest::header::HeaderMap;

use super::{GraphQLOperationContext, ResolverContextLike};
use crate::http::RequestContext;
use crate::path_value::PathValue;

// TODO: rename to ResolverContext
#[derive(Clone)]
pub struct EvaluationContext<'a, Ctx: ResolverContextLike<'a>> {
    // Context create for each GraphQL Request
    pub req_ctx: &'a RequestContext,

    // Async GraphQL Context
    // Contains current value and arguments
    pub graphql_ctx: &'a Ctx,

    // Overridden Value for Async GraphQL Context
    graphql_ctx_value: Option<Arc<Value>>,

    // Overridden Arguments for Async GraphQL Context
    graphql_ctx_args: Option<Arc<Value>>,

    // TODO: JS timeout should be read from server settings
    pub timeout: Duration,
}

impl<'a, A: ResolverContextLike<'a>> EvaluationContext<'a, A> {
    pub fn with_value(&self, value: Value) -> EvaluationContext<'a, A> {
        let mut ctx = self.clone();
        ctx.graphql_ctx_value = Some(Arc::new(value));
        ctx
    }

    pub fn with_args(&self, args: Value) -> EvaluationContext<'a, A> {
        let mut ctx = self.clone();
        ctx.graphql_ctx_args = Some(Arc::new(args));
        ctx
    }
}

impl<'a, Ctx: ResolverContextLike<'a>> EvaluationContext<'a, Ctx> {
    pub fn new(req_ctx: &'a RequestContext, graphql_ctx: &'a Ctx) -> EvaluationContext<'a, Ctx> {
        Self {
            timeout: Duration::from_millis(5),
            req_ctx,
            graphql_ctx,
            graphql_ctx_value: None,
            graphql_ctx_args: None,
        }
    }

    pub fn value(&self) -> Option<&Value> {
        self.graphql_ctx.value()
    }

    pub fn path_arg<T: AsRef<str>>(&self, path: &[T]) -> Option<Value> {
        // TODO: add unit tests for this
        if let Some(v) = &self.graphql_ctx_args {
            v.get_path_value(path)
        } else {
            self.graphql_ctx.args().and_then(|v| v.get_path_value(path))
        }
    }

    pub fn path_value<T: AsRef<str>>(&self, path: &[T]) -> Option<Value> {
        // TODO: add unit tests for this
        self.graphql_ctx_value
            .as_deref()
            .or(self.graphql_ctx.value())
            .and_then(|v| v.get_path_value(path))
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.req_ctx.request_headers
    }

    pub fn vars(&self) -> &BTreeMap<String, String> {
        &self.req_ctx.server.vars
    }

    pub fn add_error(&self, error: ServerError) {
        self.graphql_ctx.add_error(error)
    }
}

impl<'a, Ctx: ResolverContextLike<'a>> GraphQLOperationContext for EvaluationContext<'a, Ctx> {
    fn selection_set(&self) -> Option<String> {
        let selection_set = self.graphql_ctx.field()?.selection_set();

        format_selection_set(selection_set)
    }
}

fn format_selection_set<'a>(
    selection_set: impl Iterator<Item = SelectionField<'a>>,
) -> Option<String> {
    let set = selection_set
        .map(format_selection_field)
        .collect::<Vec<_>>();

    if set.is_empty() {
        return None;
    }

    Some(format!("{{ {} }}", set.join(" ")))
}

fn format_selection_field(field: SelectionField) -> String {
    let name = field.name();
    let arguments = format_selection_field_arguments(field);
    let selection_set = format_selection_set(field.selection_set());

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
            tracing::warn!("Failed to resolve arguments for field {name}, due to error: {error}");

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

impl<'ctx, Ctx: ResolverContextLike<'ctx>> PathValue for EvaluationContext<'ctx, Ctx> {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>,
    {
        path.split_first()
            .and_then(|(head, tail)| match head.as_ref() {
                "value" => self.path_value(tail),
                "args" => self.path_arg(tail),
                "headers" => self.headers().get_path_value(tail),
                "vars" => self.vars().get_path_value(tail),
                "env" => self.req_ctx.runtime.env.get_path_value(tail),
                _ => None,
            })
    }
}

#[cfg(test)]
mod tests {
    mod impl_path_value {
        use std::collections::BTreeMap;
        use std::sync::Arc;

        use async_graphql::SelectionField;
        use async_graphql_value::{ConstValue as Value, Name, Number};
        use hyper::header::HeaderValue;
        use hyper::HeaderMap;
        use indexmap::IndexMap;
        use once_cell::sync::Lazy;

        use crate::http::RequestContext;
        use crate::lambda::{EvaluationContext, ResolverContextLike};
        use crate::path_value::PathValue;
        use crate::EnvIO;

        struct Env {
            env: BTreeMap<String, String>,
        }

        impl EnvIO for Env {
            fn get(&self, key: &str) -> Option<String> {
                self.env.get(key).cloned()
            }
        }

        impl Env {
            pub fn init(map: BTreeMap<String, String>) -> Self {
                Self { env: map }
            }
        }

        static TEST_VALUES: Lazy<Value> = Lazy::new(|| {
            let mut root = IndexMap::new();
            let mut nested = IndexMap::new();

            nested.insert(
                Name::new("existing"),
                Value::String("nested-test".to_owned()),
            );
            root.insert(Name::new("bool"), Value::Boolean(true));
            root.insert(Name::new("nested"), Value::Object(nested));
            root.insert(Name::new("number"), Value::Number(Number::from(2)));
            root.insert(Name::new("str"), Value::String("str-test".to_owned()));

            Value::Object(root)
        });

        static TEST_ARGS: Lazy<IndexMap<Name, Value>> = Lazy::new(|| {
            let mut root = IndexMap::new();
            let mut nested = IndexMap::new();

            nested.insert(
                Name::new("existing"),
                Value::String("nested-test".to_owned()),
            );

            root.insert(Name::new("nested"), Value::Object(nested));
            root.insert(Name::new("root"), Value::String("root-test".to_owned()));

            root
        });

        static TEST_HEADERS: Lazy<HeaderMap> = Lazy::new(|| {
            let mut map = HeaderMap::new();

            map.insert("x-existing", HeaderValue::from_static("header"));

            map
        });

        static TEST_VARS: Lazy<BTreeMap<String, String>> = Lazy::new(|| {
            let mut map = BTreeMap::new();

            map.insert("existing".to_owned(), "var".to_owned());

            map
        });

        static TEST_ENV_VARS: Lazy<BTreeMap<String, String>> = Lazy::new(|| {
            let mut map = BTreeMap::new();

            map.insert("existing".to_owned(), "env".to_owned());

            map
        });

        #[derive(Clone)]
        struct MockGraphqlContext;

        impl<'a> ResolverContextLike<'a> for MockGraphqlContext {
            fn value(&'a self) -> Option<&'a Value> {
                Some(&TEST_VALUES)
            }

            fn args(&'a self) -> Option<&'a IndexMap<Name, Value>> {
                Some(&TEST_ARGS)
            }

            fn field(&'a self) -> Option<SelectionField> {
                None
            }

            fn add_error(&'a self, _: async_graphql::ServerError) {}
        }

        static REQ_CTX: Lazy<RequestContext> = Lazy::new(|| {
            let mut req_ctx = RequestContext::default().request_headers(TEST_HEADERS.clone());

            req_ctx.server.vars = TEST_VARS.clone();
            req_ctx.runtime.env = Arc::new(Env::init(TEST_ENV_VARS.clone()));

            req_ctx
        });

        static EVAL_CTX: Lazy<EvaluationContext<'static, MockGraphqlContext>> =
            Lazy::new(|| EvaluationContext::new(&REQ_CTX, &MockGraphqlContext));

        #[test]
        fn get_path() {
            // value
            assert_eq!(
                EVAL_CTX.get_path_value(&["value", "bool"]),
                Some(Value::Boolean(true))
            );
            assert_eq!(
                EVAL_CTX.get_path_value(&["value", "number"]),
                Some(Value::Number(Number::from(2)))
            );
            assert_eq!(
                EVAL_CTX.get_path_value(&["value", "str"]),
                Some(Value::String("str-test".to_owned()))
            );
            assert_eq!(
                EVAL_CTX.get_path_value(&["value", "nested"]),
                Some(serde_json::from_str(r#"{"existing":"nested-test"}"#).unwrap())
            );
            assert_eq!(EVAL_CTX.get_path_value(&["value", "missing"]), None);
            assert_eq!(
                EVAL_CTX.get_path_value(&["value", "nested", "missing"]),
                None
            );
            assert_eq!(
                    EVAL_CTX.get_path_value(&["value"]),
                    Some(serde_json::from_str(r#"{"bool":true,"nested":{"existing":"nested-test"},"number":2,"str":"str-test"}"#).unwrap())
                );

            // args
            assert_eq!(
                EVAL_CTX.get_path_value(&["args", "root"]),
                Some(Value::String("root-test".to_owned()))
            );
            assert_eq!(
                EVAL_CTX.get_path_value(&["args", "nested"]),
                Some(serde_json::from_str(r#"{"existing":"nested-test"}"#).unwrap())
            );
            assert_eq!(EVAL_CTX.get_path_value(&["args", "missing"]), None);
            assert_eq!(
                EVAL_CTX.get_path_value(&["args", "nested", "missing"]),
                None
            );
            assert_eq!(
                EVAL_CTX.get_path_value(&["args"]),
                Some(
                    serde_json::from_str(
                        r#"{"nested":{"existing":"nested-test"},"root":"root-test"}"#
                    )
                    .unwrap()
                )
            );

            // headers
            assert_eq!(
                EVAL_CTX.get_path_value(&["headers", "x-existing"]),
                Some(Value::String("header".to_owned()))
            );
            assert_eq!(EVAL_CTX.get_path_value(&["headers", "x-missing"]), None);

            // vars
            assert_eq!(
                EVAL_CTX.get_path_value(&["vars", "existing"]),
                Some(Value::String("var".to_owned()))
            );
            assert_eq!(EVAL_CTX.get_path_value(&["vars", "missing"]), None);
            assert_eq!(
                EVAL_CTX.get_path_value(&["vars"]),
                Some(serde_json::from_str(r#"{"existing":"var"}"#).unwrap())
            );

            // envs
            assert_eq!(
                EVAL_CTX.get_path_value(&["env", "existing"]),
                Some(Value::String("env".to_owned()))
            );
            assert_eq!(EVAL_CTX.get_path_value(&["env", "x-missing"]), None);

            // other value types
            assert_eq!(EVAL_CTX.get_path_value(&["foo", "key"]), None);
            assert_eq!(EVAL_CTX.get_path_value(&["bar", "key"]), None);
            assert_eq!(EVAL_CTX.get_path_value(&["baz", "key"]), None);
        }
    }
}
