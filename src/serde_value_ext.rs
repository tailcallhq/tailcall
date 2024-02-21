use anyhow::Result;
use async_graphql::Value as GraphQLValue;
use indexmap::IndexMap;
use serde_json::Value;

use crate::mustache::Mustache;
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
            let out = serde_json::from_str::<Value>(s.as_str());
            match out {
                Ok(v) => async_graphql::Value::from_json(v).map_err(|e| anyhow::anyhow!(e)),
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

impl ValueExt for Value {
    // Optimize render_value to avoid unnecessary conversions
    fn render_value(&self, ctx: &impl PathString) -> Result<GraphQLValue> {
        match self {
            Value::Object(obj) => {
                let mut out = IndexMap::new();
                for (k, v) in obj {
                    let value = Mustache::parse(v.to_string().as_str())?;
                    if !value.is_const() {
                        out.insert(
                            async_graphql::Name::new(k),
                            eval_types(&string_to_value(value.render(ctx)))?,
                        );
                    } else {
                        out.insert(async_graphql::Name::new(k), v.render_value(ctx)?);
                    }
                }
                Ok(async_graphql::Value::Object(out))
            }
            Value::Array(arr) => {
                let mut out = Vec::new();
                for v in arr {
                    let value = Mustache::parse(v.to_string().as_str())?;
                    if !value.is_const() {
                        out.push(eval_types(&string_to_value(value.render(ctx)))?);
                    } else {
                        out.push(v.render_value(ctx)?);
                    }
                }
                Ok(async_graphql::Value::List(out))
            }
            Value::String(str) => {
                let value = Mustache::parse(str)?;
                if !value.is_const() {
                    eval_types(&string_to_value(value.render(ctx)))
                } else {
                    Ok(async_graphql::Value::String(str.to_owned()))
                }
            }
            _ => async_graphql::Value::from_json(self.clone()).map_err(|e| anyhow::anyhow!(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::serde_value_ext::ValueExt;

    #[test]
    fn test_render_value() {
        let value = json!({"a": "{{foo}}"});
        let ctx = json!({"foo": {"bar": "baz"}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": {"bar": "baz"}})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_nested() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let ctx = json!({"foo": {"bar": {"baz": 1}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": 1})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_nested_str() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let ctx = json!({"foo": {"bar": {"baz": "foo"}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": "foo"})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_nested_bool() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let ctx = json!({"foo": {"bar": {"baz": true}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": true})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_nested_float() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let ctx = json!({"foo": {"bar": {"baz": 1.1}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": 1.1})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_arr() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let ctx = json!({"foo": {"bar": {"baz": [1,2,3]}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": [1, 2, 3]})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_arr_template() {
        let value = json!({"a": ["{{foo.bar.baz}}", "{{foo.bar.qux}}"]});
        let ctx = json!({"foo": {"bar": {"baz": 1, "qux": 2}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": [1, 2]})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }
}
