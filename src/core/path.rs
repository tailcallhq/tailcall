//! The path module provides a trait for accessing values from a JSON-like
//! structure.
use std::borrow::Cow;

use serde_json::json;

use crate::core::ir::{EvalContext, ResolverContextLike};
use crate::core::json::JsonLike;

///
/// The PathString trait provides a method for accessing values from a JSON-like
/// structure. The returned value is encoded as a plain string.
/// This is typically used in evaluating mustache templates.
pub trait PathString {
    fn path_string<'a, T: AsRef<str>>(&'a self, path: &'a [T]) -> Option<Cow<'a, str>>;
}

/// PathValue trait provides a method for accessing values from JSON-like
/// structure, the returned value is wrapped with RawValue enum, delegating
/// encoding to the client of this method.
pub trait PathValue {
    fn raw_value<'a, T: AsRef<str>>(&'a self, path: &[T]) -> Option<ValueString<'a>>;
}

///
/// The PathGraphql trait provides a method for accessing values from a
/// JSON-like structure. The returned value is encoded as a GraphQL Value.
pub trait PathGraphql {
    fn path_graphql<T: AsRef<str>>(&self, path: &[T]) -> Option<String>;
}

impl PathString for serde_json::Value {
    fn path_string<'a, T: AsRef<str>>(&'a self, path: &'a [T]) -> Option<Cow<'a, str>> {
        self.get_path(path).map(move |a| match a {
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
        Cow::Borrowed(async_graphql::Value::Enum(n)) => Some(Cow::Borrowed(n)),
        _ => None,
    }
}

///
/// An optimized version of async_graphql::Value that handles strings in a more
/// efficient manner.
#[derive(Clone, Debug, PartialEq)]
pub enum ValueString<'a> {
    Value(Cow<'a, async_graphql::Value>),
    String(Cow<'a, str>),
}

impl<Ctx: ResolverContextLike> EvalContext<'_, Ctx> {
    fn to_raw_value<T: AsRef<str>>(&self, path: &[T]) -> Option<ValueString<'_>> {
        let ctx = self;

        if path.is_empty() {
            return None;
        }

        if path.len() == 1 {
            return match path[0].as_ref() {
                "value" => Some(ValueString::Value(ctx.path_value(&[] as &[T])?)),
                "args" => Some(ValueString::Value(ctx.path_arg::<&str>(&[])?)),
                "vars" => Some(ValueString::String(Cow::Owned(
                    json!(ctx.vars()).to_string(),
                ))),
                _ => None,
            };
        }

        path.split_first()
            .and_then(move |(head, tail)| match head.as_ref() {
                "value" => Some(ValueString::Value(ctx.path_value(tail)?)),
                "args" => Some(ValueString::Value(ctx.path_arg(tail)?)),
                "headers" => Some(ValueString::String(Cow::Borrowed(
                    ctx.header(tail[0].as_ref())?,
                ))),
                "vars" => Some(ValueString::String(Cow::Borrowed(
                    ctx.var(tail[0].as_ref())?,
                ))),
                "env" => Some(ValueString::String(ctx.env_var(tail[0].as_ref())?)),
                _ => None,
            })
    }
}

impl<Ctx: ResolverContextLike> PathValue for EvalContext<'_, Ctx> {
    fn raw_value<'b, T: AsRef<str>>(&'b self, path: &[T]) -> Option<ValueString<'b>> {
        self.to_raw_value(path)
    }
}

impl<Ctx: ResolverContextLike> PathString for EvalContext<'_, Ctx> {
    fn path_string<T: AsRef<str>>(&self, path: &[T]) -> Option<Cow<'_, str>> {
        self.to_raw_value(path).and_then(|value| match value {
            ValueString::String(env) => Some(env),
            ValueString::Value(value) => convert_value(value),
        })
    }
}

impl<Ctx: ResolverContextLike> PathGraphql for EvalContext<'_, Ctx> {
    fn path_graphql<T: AsRef<str>>(&self, path: &[T]) -> Option<String> {
        if path.len() < 2 {
            return None;
        }

        self.to_raw_value(path).map(|value| match value {
            ValueString::Value(val) => val.to_string(),
            ValueString::String(val) => format!(r#""{val}""#),
        })
    }
}

#[cfg(test)]
mod tests {

    mod evaluation_context {
        use std::borrow::Cow;
        use std::collections::BTreeMap;
        use std::sync::Arc;

        use async_graphql_value::{ConstValue as Value, Name, Number};
        use http::header::{HeaderMap, HeaderValue};
        use indexmap::IndexMap;
        use once_cell::sync::Lazy;

        use crate::core::http::RequestContext;
        use crate::core::ir::{EvalContext, ResolverContextLike, SelectionField};
        use crate::core::path::{PathGraphql, PathString, PathValue, ValueString};
        use crate::core::EnvIO;

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

        impl ResolverContextLike for MockGraphqlContext {
            fn value(&self) -> Option<&Value> {
                Some(&TEST_VALUES)
            }

            fn args(&self) -> Option<&IndexMap<Name, Value>> {
                Some(&TEST_ARGS)
            }

            fn field(&self) -> Option<SelectionField> {
                None
            }

            fn is_query(&self) -> bool {
                false
            }

            fn add_error(&self, _: async_graphql::ServerError) {}
        }

        static REQ_CTX: Lazy<RequestContext> = Lazy::new(|| {
            let mut req_ctx = RequestContext::default().allowed_headers(TEST_HEADERS.clone());

            req_ctx.server.vars = TEST_VARS.clone();
            req_ctx.runtime.env = Arc::new(Env::init(TEST_ENV_VARS.clone()));

            req_ctx
        });

        static EVAL_CTX: Lazy<EvalContext<'static, MockGraphqlContext>> =
            Lazy::new(|| EvalContext::new(&REQ_CTX, &MockGraphqlContext));

        #[test]
        fn path_to_value() {
            let mut map = IndexMap::default();
            map.insert(
                async_graphql_value::Name::new("number"),
                async_graphql::Value::Number(2.into()),
            );
            map.insert(
                async_graphql_value::Name::new("str"),
                async_graphql::Value::String("str-test".into()),
            );
            map.insert(
                async_graphql_value::Name::new("bool"),
                async_graphql::Value::Boolean(true),
            );
            let mut nested_map = IndexMap::default();
            nested_map.insert(
                async_graphql_value::Name::new("existing"),
                async_graphql::Value::String("nested-test".into()),
            );
            map.insert(
                async_graphql_value::Name::new("nested"),
                async_graphql::Value::Object(nested_map),
            );

            // value
            assert_eq!(
                EVAL_CTX.raw_value(&["value", "bool"]),
                Some(ValueString::Value(Cow::Borrowed(
                    &async_graphql::Value::Boolean(true)
                )))
            );
            assert_eq!(
                EVAL_CTX.raw_value(&["value", "number"]),
                Some(ValueString::Value(Cow::Borrowed(
                    &async_graphql::Value::Number(2.into())
                )))
            );
            assert_eq!(
                EVAL_CTX.raw_value(&["value", "str"]),
                Some(ValueString::Value(Cow::Borrowed(
                    &async_graphql::Value::String("str-test".into())
                )))
            );
            assert_eq!(EVAL_CTX.raw_value(&["value", "missing"]), None);
            assert_eq!(EVAL_CTX.raw_value(&["value", "nested", "missing"]), None);
            assert_eq!(
                EVAL_CTX.raw_value(&["value"]),
                Some(ValueString::Value(Cow::Borrowed(
                    &async_graphql::Value::Object(map.clone()),
                )))
            );

            // args
            assert_eq!(
                EVAL_CTX.raw_value(&["args", "root"]),
                Some(ValueString::Value(Cow::Borrowed(
                    &async_graphql::Value::String("root-test".into()),
                )))
            );

            let mut expected = IndexMap::new();
            expected.insert(
                async_graphql_value::Name::new("existing"),
                async_graphql::Value::String("nested-test".into()),
            );
            assert_eq!(
                EVAL_CTX.raw_value(&["args", "nested"]),
                Some(ValueString::Value(Cow::Borrowed(
                    &async_graphql::Value::Object(expected)
                )))
            );

            assert_eq!(EVAL_CTX.raw_value(&["args", "missing"]), None);
            assert_eq!(EVAL_CTX.raw_value(&["args", "nested", "missing"]), None);

            let mut expected = IndexMap::new();
            let mut nested_map = IndexMap::new();
            nested_map.insert(
                async_graphql_value::Name::new("existing"),
                async_graphql::Value::String("nested-test".into()),
            );
            expected.insert(
                async_graphql_value::Name::new("nested"),
                async_graphql::Value::Object(nested_map),
            );
            expected.insert(
                async_graphql_value::Name::new("root"),
                async_graphql::Value::String("root-test".into()),
            );
            assert_eq!(
                EVAL_CTX.raw_value(&["args"]),
                Some(ValueString::Value(Cow::Borrowed(
                    &async_graphql::Value::Object(expected)
                )))
            );

            // headers
            assert_eq!(
                EVAL_CTX.raw_value(&["headers", "x-existing"]),
                Some(ValueString::String(Cow::Borrowed("header")))
            );
            assert_eq!(EVAL_CTX.raw_value(&["headers", "x-missing"]), None);

            // vars
            assert_eq!(
                EVAL_CTX.raw_value(&["vars", "existing"]),
                Some(ValueString::String(Cow::Borrowed("var")))
            );
            assert_eq!(EVAL_CTX.raw_value(&["vars", "missing"]), None);
            assert_eq!(
                EVAL_CTX.raw_value(&["vars"]),
                Some(ValueString::String(Cow::Borrowed(r#"{"existing":"var"}"#)))
            );

            // envs
            assert_eq!(
                EVAL_CTX.raw_value(&["env", "existing"]),
                Some(ValueString::String(Cow::Borrowed("env")))
            );
            assert_eq!(EVAL_CTX.raw_value(&["env", "x-missing"]), None);

            // other value types
            assert_eq!(EVAL_CTX.raw_value(&["foo", "key"]), None);
            assert_eq!(EVAL_CTX.raw_value(&["bar", "key"]), None);
            assert_eq!(EVAL_CTX.raw_value(&["baz", "key"]), None);
        }

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
