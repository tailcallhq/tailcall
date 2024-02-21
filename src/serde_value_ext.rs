use std::borrow::Cow;

use anyhow::{anyhow, Result};
use async_graphql::{Name, Value as GraphQLValue};
use indexmap::IndexMap;
use serde_json::Value;

use crate::blueprint::{MustacheOrValue, ValueOrDynamic};
use crate::path::PathString;

pub trait ValueExt {
    fn render_value(&self, ctx: &impl PathString) -> Result<GraphQLValue>;
}

fn eval_types(value: &Value) -> Result<GraphQLValue> {
    match value {
        Value::Array(arr) => {
            let out: Result<Vec<_>> = arr.iter().map(eval_types).collect();
            out.map(GraphQLValue::List)
        }
        Value::Object(obj) => {
            let out: Result<IndexMap<_, _>> = obj
                .iter()
                .map(|(k, v)| {
                    let key: Cow<'_, str> = Cow::Borrowed(k);
                    eval_types(v).map(|val| (Name::new(key), val))
                })
                .collect();
            out.map(GraphQLValue::Object)
        }
        Value::String(s) => serde_json::from_str::<Value>(s)
            .map_err(anyhow::Error::new)
            .and_then(|a| eval_types(&a))
            .or_else(|_| Ok(GraphQLValue::String(Cow::Borrowed(s).into_owned()))),
        _ => GraphQLValue::from_json(value.clone()).map_err(|e| anyhow!(e)),
    }
}

impl ValueExt for ValueOrDynamic {
    fn render_value<'a>(&self, ctx: &'a impl PathString) -> Result<GraphQLValue> {
        match self {
            ValueOrDynamic::Value(value) => eval_types(value),
            ValueOrDynamic::Mustache(m) => {
                let rendered: Cow<'a, str> = Cow::Owned(m.render(ctx));
                serde_json::from_str::<Value>(rendered.as_ref())
                    .map_err(anyhow::Error::new)
                    .and_then(|a| eval_types(&a))
                    .or_else(|_| Ok(GraphQLValue::String(rendered.into_owned())))
            }
            ValueOrDynamic::MustacheObject(obj) => {
                let out: Result<IndexMap<_, _>> = obj
                    .iter()
                    .map(|(k, v)| {
                        let key: Cow<'_, str> = Cow::Borrowed(k);
                        match v {
                            MustacheOrValue::Value(value) => eval_types(value),
                            MustacheOrValue::Mustache(m) => {
                                let rendered: Cow<'a, str> = Cow::Owned(m.render(ctx));
                                serde_json::from_str::<Value>(rendered.as_ref())
                                    .map_err(anyhow::Error::new)
                                    .and_then(|a| eval_types(&a))
                                    .or_else(|_| Ok(GraphQLValue::String(rendered.into_owned())))
                            }
                        }
                            .map(|val| (Name::new(&key), val))
                    })
                    .collect();
                out.map(GraphQLValue::Object)
            }
            ValueOrDynamic::MustacheArray(arr) => {
                let out: Result<Vec<_>> = arr
                    .iter()
                    .map(|v| match v {
                        MustacheOrValue::Value(value) => eval_types(value),
                        MustacheOrValue::Mustache(m) => {
                            let rendered: Cow<'a, str> = Cow::Owned(m.render(ctx));
                            serde_json::from_str::<Value>(&rendered)
                                .map_err(anyhow::Error::new)
                                .and_then(|a| eval_types(&a))
                                .or_else(|_| Ok(GraphQLValue::String(rendered.into_owned())))
                        }
                    })
                    .collect();
                out.map(GraphQLValue::List)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::blueprint::ValueOrDynamic;
    use crate::serde_value_ext::ValueExt;

    #[test]
    fn test_render_value() {
        let value = json!({"a": "{{foo}}"});
        let value = ValueOrDynamic::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": "baz"}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": {"bar": "baz"}})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_nested() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let value = ValueOrDynamic::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": 1}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": 1})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_nested_str() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let value = ValueOrDynamic::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": "foo"}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": "foo"})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_nested_bool() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let value = ValueOrDynamic::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": true}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": true})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_nested_float() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let value = ValueOrDynamic::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": 1.1}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": 1.1})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_arr() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let value = ValueOrDynamic::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": [1,2,3]}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": [1, 2, 3]})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_arr_template() {
        let value = json!({"a": ["{{foo.bar.baz}}", "{{foo.bar.qux}}"]});
        let value = ValueOrDynamic::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": 1, "qux": 2}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": [1, 2]})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_mustache_or_value_is_const() {
        let value = json!("{{foo}}");
        let value = ValueOrDynamic::try_from(&value).unwrap();
        let ctx = json!({"foo": "bar"});
        let result = value.render_value(&ctx).unwrap();
        let expected = async_graphql::Value::String("bar".to_owned());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mustache_arr_obj() {
        let value = json!([{"a": "{{foo.bar.baz}}"}, {"a": "{{foo.bar.qux}}"}]);
        let value = ValueOrDynamic::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": 1, "qux": 2}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!([{"a": 1}, {"a":2}])).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_mustache_arr_obj_arr() {
        let value = json!([{"a": [{"aa": "{{foo.bar.baz}}"}]}, {"a": [{"aa": "{{foo.bar.qux}}"}]}]);
        let value = ValueOrDynamic::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": 1, "qux": 2}}});
        let result = value.render_value(&ctx);
        let expected =
            async_graphql::Value::from_json(json!([{"a": [{"aa": 1}]}, {"a":[{"aa": 2}]}]))
                .unwrap();
        assert_eq!(result.unwrap(), expected);
    }
}
