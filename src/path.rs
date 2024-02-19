use std::borrow::Cow;

use indexmap::IndexMap;

use crate::json::JsonLike;
use crate::lambda::{EvaluationContext, ResolverContextLike};

///
/// The path module provides a trait for accessing values from a JSON-like structure.
///

///
/// The PathString trait provides a method for accessing values from a JSON-like structure.
/// The returned value is encoded as a plain string.
/// This is typically used in evaluating mustache templates.
///
pub trait PathString {
    fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, async_graphql::Value>>;
}

///
/// The PathGraphql trait provides a method for accessing values from a JSON-like structure.
/// The returned value is encoded as a GraphQL Value.
///
pub trait PathGraphql {
    fn path_graphql<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, async_graphql::Value>>;
}

impl PathString for serde_json::Value {
    fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, async_graphql::Value>> {
        self.get_path(path).and_then(|a| match a {
            serde_json::Value::String(s) => {
                Some(Cow::Owned(async_graphql::Value::String(s.to_string())))
            }
            _ => Some(Cow::Owned(
                async_graphql::Value::from_json(a.to_owned()).ok()?,
            )),
        })
    }
}

impl<'a, Ctx: ResolverContextLike<'a>> PathString for EvaluationContext<'a, Ctx> {
    fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, async_graphql::Value>> {
        let ctx = self;

        if path.is_empty() {
            return None;
        }

        if path.len() == 1 {
            return match path[0].as_ref() {
                "value" => Some(Cow::Borrowed(ctx.path_value(&[] as &[T])?)),
                "args" => Some(Cow::Owned(
                    ctx.graphql_ctx
                        .args()
                        .map(|v| async_graphql::Value::Object(v.to_owned()))?,
                )),
                "vars" => Some(Cow::Owned(async_graphql::Value::Object(
                    ctx.vars()
                        .iter()
                        .map(|(k, v)| {
                            (
                                async_graphql::Name::new(k),
                                async_graphql::Value::String(v.clone()),
                            )
                        })
                        .collect::<IndexMap<_, _>>(),
                ))),
                _ => None,
            };
        }

        path.split_first()
            .and_then(|(head, tail)| match head.as_ref() {
                "value" => Some(Cow::Borrowed(ctx.path_value(tail)?)),
                "args" => ctx.arg(tail).map(Cow::Borrowed),
                "headers" => ctx
                    .header(tail[0].as_ref())
                    .map(|v| Cow::Owned(async_graphql::Value::String(v.to_string()))),
                "vars" => ctx
                    .var(tail[0].as_ref())
                    .map(|v| Cow::Owned(async_graphql::Value::String(v.to_string()))),
                "env" => ctx
                    .env_var(tail[0].as_ref())
                    .map(|v| Cow::Owned(async_graphql::Value::String(v))),
                _ => None,
            })
    }
}

impl<'a, Ctx: ResolverContextLike<'a>> PathGraphql for EvaluationContext<'a, Ctx> {
    fn path_graphql<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, async_graphql::Value>> {
        let ctx = self;

        if path.len() < 2 {
            return None;
        }

        path.split_first()
            .and_then(|(head, tail)| match head.as_ref() {
                "value" => Some(Cow::Borrowed(ctx.path_value(tail)?)),
                "args" => ctx.arg(tail).map(Cow::Borrowed),
                "headers" => ctx
                    .header(tail[0].as_ref())
                    .map(|v| Cow::Owned(async_graphql::Value::String(v.to_string()))),
                "vars" => ctx
                    .var(tail[0].as_ref())
                    .map(|v| Cow::Owned(async_graphql::Value::String(v.to_string()))),
                "env" => ctx
                    .env_var(tail[0].as_ref())
                    .map(|v| Cow::Owned(async_graphql::Value::String(v))),
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

        use async_graphql::{ScalarType, SelectionField};
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
            let mut req_ctx = RequestContext::default().req_headers(TEST_HEADERS.clone());

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
                Some(Cow::Owned(true.to_value()))
            );
            assert_eq!(
                EVAL_CTX.path_string(&["value", "number"]),
                Some(Cow::Owned(2.to_value()))
            );
            assert_eq!(
                EVAL_CTX.path_string(&["value", "str"]),
                Some(Cow::Owned("str-test".to_string().to_value()))
            );
            assert_eq!(
                EVAL_CTX.path_string(&["value", "nested"]),
                Some(Cow::Owned(
                    async_graphql::Value::from_json(serde_json::json!({"existing": "nested-test"}))
                        .unwrap()
                ))
            );
            assert_eq!(EVAL_CTX.path_string(&["value", "missing"]), None);
            assert_eq!(EVAL_CTX.path_string(&["value", "nested", "missing"]), None);
            assert_eq!(
                EVAL_CTX.path_string(&["value"]),
                Some(Cow::Owned(
                    async_graphql::Value::from_json(serde_json::json!({
                        "bool": true,
                        "number": 2,
                        "str": "str-test",
                        "nested": {"existing": "nested-test"}
                    }))
                    .unwrap()
                ))
            );

            // // args
            assert_eq!(
                EVAL_CTX.path_string(&["args", "root"]),
                Some(Cow::Owned("root-test".to_string().to_value()))
            );
            assert_eq!(
                EVAL_CTX.path_string(&["args", "nested"]),
                Some(Cow::Owned(
                    async_graphql::Value::from_json(serde_json::json!({"existing": "nested-test"}))
                        .unwrap()
                ))
            );
            assert_eq!(EVAL_CTX.path_string(&["args", "missing"]), None);
            assert_eq!(EVAL_CTX.path_string(&["args", "nested", "missing"]), None);
            assert_eq!(
                EVAL_CTX.path_string(&["args"]),
                Some(Cow::Owned(
                    async_graphql::Value::from_json(serde_json::json!({
                        "root": "root-test",
                        "nested": {"existing": "nested-test"}
                    }))
                    .unwrap()
                ))
            );

            // // headers
            assert_eq!(
                EVAL_CTX.path_string(&["headers", "x-existing"]),
                Some(Cow::Owned("header".to_string().to_value()))
            );
            assert_eq!(EVAL_CTX.path_string(&["headers", "x-missing"]), None);

            // vars
            assert_eq!(
                EVAL_CTX.path_string(&["vars", "existing"]),
                Some(Cow::Owned("var".to_string().to_value()))
            );
            assert_eq!(EVAL_CTX.path_string(&["vars", "missing"]), None);
            assert_eq!(
                EVAL_CTX.path_string(&["vars"]),
                Some(Cow::Owned(
                    async_graphql::Value::from_json(serde_json::json!({"existing": "var"}))
                        .unwrap()
                ))
            );

            // envs
            assert_eq!(
                EVAL_CTX.path_string(&["env", "existing"]),
                Some(Cow::Owned("env".to_string().to_value()))
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
                EVAL_CTX
                    .path_graphql(&["value", "bool"])
                    .map(|v| v.to_string()),
                Some("true".to_owned())
            );
            assert_eq!(
                EVAL_CTX
                    .path_graphql(&["value", "number"])
                    .map(|v| v.to_string()),
                Some("2".to_owned())
            );
            assert_eq!(
                EVAL_CTX
                    .path_graphql(&["value", "str"])
                    .map(|v| v.to_string()),
                Some("\"str-test\"".to_owned())
            );
            assert_eq!(
                EVAL_CTX
                    .path_graphql(&["value", "nested"])
                    .map(|v| v.to_string()),
                Some("{existing: \"nested-test\"}".to_owned())
            );
            assert_eq!(EVAL_CTX.path_graphql(&["value", "missing"]), None);
            assert_eq!(EVAL_CTX.path_graphql(&["value", "nested", "missing"]), None);

            // args
            assert_eq!(
                EVAL_CTX
                    .path_graphql(&["args", "root"])
                    .map(|v| v.to_string()),
                Some("\"root-test\"".to_owned())
            );
            assert_eq!(
                EVAL_CTX
                    .path_graphql(&["args", "nested"])
                    .map(|v| v.to_string()),
                Some("{existing: \"nested-test\"}".to_owned())
            );
            assert_eq!(EVAL_CTX.path_graphql(&["args", "missing"]), None);
            assert_eq!(EVAL_CTX.path_graphql(&["args", "nested", "missing"]), None);

            // headers
            assert_eq!(
                EVAL_CTX
                    .path_graphql(&["headers", "x-existing"])
                    .map(|v| v.to_string()),
                Some("\"header\"".to_owned())
            );
            assert_eq!(EVAL_CTX.path_graphql(&["headers", "x-missing"]), None);

            // vars
            assert_eq!(
                EVAL_CTX
                    .path_graphql(&["vars", "existing"])
                    .map(|v| v.to_string()),
                Some("\"var\"".to_owned())
            );
            assert_eq!(EVAL_CTX.path_graphql(&["vars", "missing"]), None);

            // envs
            assert_eq!(
                EVAL_CTX
                    .path_graphql(&["env", "existing"])
                    .map(|v| v.to_string()),
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
