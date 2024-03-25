use crate::{
    config::ConfigReaderContext,
    json::JsonLike,
    lambda::{EvaluationContext, ResolverContextLike},
};

pub trait PathValue {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>;
}

impl<'ctx, Ctx: ResolverContextLike<'ctx>> PathValue for EvaluationContext<'ctx, Ctx> {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>,
    {
        if path.is_empty() {
            return None;
        }

        if path.len() == 1 {
            return match path[0].as_ref() {
                "value" => Some(self.path_value(&[] as &[Path])?.into_owned()),
                "args" => self
                    .graphql_ctx
                    .args()
                    .map(|map| async_graphql::Value::Object(map.clone())),
                "vars" => Some(async_graphql::Value::Object(indexmap::IndexMap::from_iter(
                    self.vars()
                        .iter()
                        .map(|(k, v)| (async_graphql::Name::new(k), async_graphql::Value::from(v))),
                ))),
                _ => None,
            };
        }

        let tail = &path[1..];

        match path[0].as_ref() {
            "value" => Some(self.path_value(tail)?.into_owned()),
            "args" => Some(self.path_arg(tail)?.into_owned()),
            "headers" => self
                .header(tail[0].as_ref())
                .map(|v| async_graphql::Value::String(v.to_string())),
            "vars" => self
                .var(tail[0].as_ref())
                .map(|v| async_graphql::Value::String(v.to_string())),
            "env" => self
                .env_var(tail[0].as_ref())
                .map(|v| async_graphql::Value::String(v.to_string())),
            _ => None,
        }
    }
}

impl<'a> PathValue for ConfigReaderContext<'a> {
    fn get_path_value<'out, Path>(&'out self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>,
    {
        if path.is_empty() {
            return None;
        }

        path.split_first()
            .and_then(|(head, tail)| match head.as_ref() {
                "vars" => self.vars.get(tail[0].as_ref()).map(|v| v.into()),
                "env" => self.env.get(tail[0].as_ref()).map(|v| v.into()),
                _ => None,
            })
    }
}

impl PathValue for serde_json::Value {
    fn get_path_value<Path>(&self, path: &[Path]) -> Option<async_graphql::Value>
    where
        Path: AsRef<str>,
    {
        self.get_path(path)
            .and_then(|v| async_graphql::Value::from_json(v.to_owned()).ok())
    }
}

#[cfg(test)]
mod tests {

    mod evaluation_context {
        use std::collections::BTreeMap;
        use std::sync::Arc;

        use async_graphql::SelectionField;
        use async_graphql_value::{ConstValue as Value, Name, Number};
        use hyper::header::HeaderValue;
        use hyper::HeaderMap;
        use indexmap::IndexMap;
        use once_cell::sync::Lazy;

        use crate::lambda::{EvaluationContext, ResolverContextLike};
        use crate::EnvIO;
        use crate::{http::RequestContext, path_value::PathValue};

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
