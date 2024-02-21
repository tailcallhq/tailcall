use anyhow::Result;
use async_graphql::Value as GraphQLValue;
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
            let mut out = Vec::new();
            for v in arr {
                let value = eval_types(v)?;
                out.push(value);
            }
            Ok(async_graphql::Value::List(out))
        }
        Value::String(s) => {
            let out = serde_json::from_str::<GraphQLValue>(s.as_str());
            match out {
                Ok(v) => Ok(v),
                Err(_) => Ok(async_graphql::Value::String(s.to_owned())),
            }
        }
        _ => async_graphql::Value::from_json(value.clone()).map_err(|e| anyhow::anyhow!(e)),
    }
}

fn string_to_value(s: String) -> Value {
    let out = serde_json::from_str::<Value>(&s);
    match out {
        Ok(v) => v,
        Err(_) => Value::String(s),
    }
}

impl ValueExt for ValueOrDynamic {
    fn render_value(&self, ctx: &impl PathString) -> Result<GraphQLValue> {
        match self {
            ValueOrDynamic::Value(value) => eval_types(value),
            ValueOrDynamic::Mustache(m) => {
                let s = m.render(ctx);
                let value = string_to_value(s);
                eval_types(&value)
            }
            ValueOrDynamic::MustacheObject(obj) => {
                let mut out = IndexMap::new();
                for (k, v) in obj {
                    match v {
                        MustacheOrValue::Value(value) => {
                            let value = eval_types(value)?;
                            out.insert(k.clone(), value);
                        }
                        MustacheOrValue::Mustache(m) => {
                            let s = m.render(ctx);
                            let value = string_to_value(s);
                            let value = eval_types(&value)?;
                            out.insert(k.clone(), value);
                        }
                    }
                }
                Ok(async_graphql::Value::Object(out))
            }
            ValueOrDynamic::MustacheArray(arr) => {
                let mut out = Vec::new();
                for v in arr {
                    match v {
                        MustacheOrValue::Value(value) => {
                            let value = eval_types(value)?;
                            out.push(value);
                        }
                        MustacheOrValue::Mustache(m) => {
                            let s = m.render(ctx);
                            let value = string_to_value(s);
                            let value = eval_types(&value)?;
                            out.push(value);
                        }
                    }
                }
                Ok(async_graphql::Value::List(out))
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
}
