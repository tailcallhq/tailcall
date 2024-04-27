use std::borrow::Cow;

use serde::{Serialize, Serializer};
use serde_json::json;

use crate::json::JsonLike;
use crate::lambda::{EvaluationContext, ResolverContextLike};

///
/// The path module provides a trait for accessing values from a JSON-like
/// structure.

///
/// The PathString trait provides a method for accessing values from a JSON-like
/// structure. The returned value is encoded as a plain string.
/// This is typically used in evaluating mustache templates.
pub trait PathString: Serialize {
    fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, str>>;
}

///
/// The PathGraphql trait provides a method for accessing values from a
/// JSON-like structure. The returned value is encoded as a GraphQL Value.
pub trait PathGraphql {
    fn path_graphql<T: AsRef<str>>(&self, path: &[T]) -> Option<String>;
}

impl PathString for serde_json::Value {
    fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, str>> {
        self.get_path(path).map(|a| match a {
            serde_json::Value::String(s) => Cow::Borrowed(s.as_str()),
            _ => Cow::Owned(a.to_string()),
        })
    }
}

fn convert_value(value: Cow<'_, async_graphql::Value>) -> Option<Cow<'_, str>> {
    match value {
        Cow::Owned(async_graphql::Value::String(s)) => Some(Cow::Owned(s)),
        Cow::Owned(async_graphql::Value::Number(n)) => Some(Cow::Owned(n.to_string())),
        Cow::Owned(async_graphql::Value::Boolean(b)) => Some(Cow::Owned(b.to_string())),
        Cow::Owned(async_graphql::Value::Object(map)) => Some(json!(map).to_string().into()),
        Cow::Owned(async_graphql::Value::List(list)) => Some(json!(list).to_string().into()),
        Cow::Borrowed(async_graphql::Value::String(s)) => Some(Cow::Borrowed(s.as_str())),
        Cow::Borrowed(async_graphql::Value::Number(n)) => Some(Cow::Owned(n.to_string())),
        Cow::Borrowed(async_graphql::Value::Boolean(b)) => Some(Cow::Owned(b.to_string())),
        Cow::Borrowed(async_graphql::Value::Object(map)) => Some(json!(map).to_string().into()),
        Cow::Borrowed(async_graphql::Value::List(list)) => Some(json!(list).to_string().into()),
        _ => None,
    }
}

impl<'a, Ctx: ResolverContextLike<'a>> Serialize for EvaluationContext<'a, Ctx> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value()
            .cloned()
            .unwrap_or_default()
            .serialize(serializer)
    }
}

impl<'a, Ctx: ResolverContextLike<'a>> PathString for EvaluationContext<'a, Ctx> {
    fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, str>> {
        let ctx = self;

        if path.is_empty() {
            return None;
        }

        if path.len() == 1 {
            return match path[0].as_ref() {
                "value" => convert_value(ctx.path_value(&[] as &[T])?),
                "args" => Some(json!(ctx.path_arg::<&str>(&[])?).to_string().into()),
                "vars" => Some(json!(ctx.vars()).to_string().into()),
                _ => None,
            };
        }

        path.split_first()
            .and_then(move |(head, tail)| match head.as_ref() {
                "value" => convert_value(ctx.path_value(tail)?),
                "args" => convert_value(ctx.path_arg(tail)?),
                "headers" => ctx.header(tail[0].as_ref()).map(|v| v.into()),
                "vars" => ctx.var(tail[0].as_ref()).map(|v| v.into()),
                "env" => ctx.env_var(tail[0].as_ref()),
                _ => None,
            })
    }
}

impl<'a, Ctx: ResolverContextLike<'a>> PathGraphql for EvaluationContext<'a, Ctx> {
    fn path_graphql<T: AsRef<str>>(&self, path: &[T]) -> Option<String> {
        let ctx = self;

        if path.len() < 2 {
            return None;
        }

        path.split_first()
            .and_then(|(head, tail)| match head.as_ref() {
                "value" => Some(ctx.path_value(tail)?.to_string()),
                "args" => Some(ctx.path_arg(tail)?.to_string()),
                "headers" => ctx.header(tail[0].as_ref()).map(|v| format!(r#""{v}""#)),
                "vars" => ctx.var(tail[0].as_ref()).map(|v| format!(r#""{v}""#)),
                "env" => ctx.env_var(tail[0].as_ref()).map(|v| format!(r#""{v}""#)),
                _ => None,
            })
    }
}

#[cfg(test)]
mod tests {

    mod evaluation_context {
        use std::borrow::Cow;
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
        use crate::path::{PathGraphql, PathString};
        use crate::EnvIO;

        struct Env {
            env: BTreeMap<String, String>,
        }

        impl EnvIO for Env {
            fn get(&self, key: &str) -> Option<Cow<'_, str>> {
                self.env.get(key).map(Cow::from)
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
            let mut req_ctx = RequestContext::default().allowed_headers(TEST_HEADERS.clone());

            req_ctx.server.vars = TEST_VARS.clone();
            req_ctx.runtime.env = Arc::new(Env::init(TEST_ENV_VARS.clone()));

            req_ctx
        });

        static EVAL_CTX: Lazy<EvaluationContext<'static, MockGraphqlContext>> =
            Lazy::new(|| EvaluationContext::new(&REQ_CTX, &MockGraphqlContext));

        #[test]
        fn path_to_string() {
            // value
            assert_eq!(
                EVAL_CTX.path_string(&["value", "bool"]),
                Some(Cow::Borrowed("true"))
            );
            assert_eq!(
                EVAL_CTX.path_string(&["value", "number"]),
                Some(Cow::Borrowed("2"))
            );
            assert_eq!(
                EVAL_CTX.path_string(&["value", "str"]),
                Some(Cow::Borrowed("str-test"))
            );
            assert_eq!(
                EVAL_CTX.path_string(&["value", "nested"]),
                Some(Cow::Borrowed("{\"existing\":\"nested-test\"}"))
            );
            assert_eq!(EVAL_CTX.path_string(&["value", "missing"]), None);
            assert_eq!(EVAL_CTX.path_string(&["value", "nested", "missing"]), None);
            assert_eq!(
                EVAL_CTX.path_string(&["value"]),
                Some(Cow::Borrowed(
                    r#"{"bool":true,"nested":{"existing":"nested-test"},"number":2,"str":"str-test"}"#
                ))
            );

            // args
            assert_eq!(
                EVAL_CTX.path_string(&["args", "root"]),
                Some(Cow::Borrowed("root-test"))
            );
            assert_eq!(
                EVAL_CTX.path_string(&["args", "nested"]),
                Some(Cow::Borrowed("{\"existing\":\"nested-test\"}"))
            );
            assert_eq!(EVAL_CTX.path_string(&["args", "missing"]), None);
            assert_eq!(EVAL_CTX.path_string(&["args", "nested", "missing"]), None);
            assert_eq!(
                EVAL_CTX.path_string(&["args"]),
                Some(Cow::Borrowed(
                    r#"{"nested":{"existing":"nested-test"},"root":"root-test"}"#
                ))
            );

            // headers
            assert_eq!(
                EVAL_CTX.path_string(&["headers", "x-existing"]),
                Some(Cow::Borrowed("header"))
            );
            assert_eq!(EVAL_CTX.path_string(&["headers", "x-missing"]), None);

            // vars
            assert_eq!(
                EVAL_CTX.path_string(&["vars", "existing"]),
                Some(Cow::Borrowed("var"))
            );
            assert_eq!(EVAL_CTX.path_string(&["vars", "missing"]), None);
            assert_eq!(
                EVAL_CTX.path_string(&["vars"]),
                Some(Cow::Borrowed(r#"{"existing":"var"}"#))
            );

            // envs
            assert_eq!(
                EVAL_CTX.path_string(&["env", "existing"]),
                Some(Cow::Borrowed("env"))
            );
            assert_eq!(EVAL_CTX.path_string(&["env", "x-missing"]), None);

            // other value types
            assert_eq!(EVAL_CTX.path_string(&["foo", "key"]), None);
            assert_eq!(EVAL_CTX.path_string(&["bar", "key"]), None);
            assert_eq!(EVAL_CTX.path_string(&["baz", "key"]), None);
        }

        #[test]
        fn path_to_graphql_string() {
            // value
            assert_eq!(
                EVAL_CTX.path_graphql(&["value", "bool"]),
                Some("true".to_owned())
            );
            assert_eq!(
                EVAL_CTX.path_graphql(&["value", "number"]),
                Some("2".to_owned())
            );
            assert_eq!(
                EVAL_CTX.path_graphql(&["value", "str"]),
                Some("\"str-test\"".to_owned())
            );
            assert_eq!(
                EVAL_CTX.path_graphql(&["value", "nested"]),
                Some("{existing: \"nested-test\"}".to_owned())
            );
            assert_eq!(EVAL_CTX.path_graphql(&["value", "missing"]), None);
            assert_eq!(EVAL_CTX.path_graphql(&["value", "nested", "missing"]), None);

            // args
            assert_eq!(
                EVAL_CTX.path_graphql(&["args", "root"]),
                Some("\"root-test\"".to_owned())
            );
            assert_eq!(
                EVAL_CTX.path_graphql(&["args", "nested"]),
                Some("{existing: \"nested-test\"}".to_owned())
            );
            assert_eq!(EVAL_CTX.path_graphql(&["args", "missing"]), None);
            assert_eq!(EVAL_CTX.path_graphql(&["args", "nested", "missing"]), None);

            // headers
            assert_eq!(
                EVAL_CTX.path_graphql(&["headers", "x-existing"]),
                Some("\"header\"".to_owned())
            );
            assert_eq!(EVAL_CTX.path_graphql(&["headers", "x-missing"]), None);

            // vars
            assert_eq!(
                EVAL_CTX.path_graphql(&["vars", "existing"]),
                Some("\"var\"".to_owned())
            );
            assert_eq!(EVAL_CTX.path_graphql(&["vars", "missing"]), None);

            // envs
            assert_eq!(
                EVAL_CTX.path_graphql(&["env", "existing"]),
                Some("\"env\"".to_owned())
            );
            assert_eq!(EVAL_CTX.path_graphql(&["env", "x-missing"]), None);

            // other value types
            assert_eq!(EVAL_CTX.path_graphql(&["foo", "key"]), None);
            assert_eq!(EVAL_CTX.path_graphql(&["bar", "key"]), None);
            assert_eq!(EVAL_CTX.path_graphql(&["baz", "key"]), None);
        }
    }
}
