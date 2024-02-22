use std::borrow::Cow;

use anyhow::Result;
use async_graphql::{Name, Value as GraphQLValue};
use indexmap::IndexMap;

use crate::blueprint::DynamicValue;
use crate::path::PathString;

pub trait ValueExt {
    fn render_value(&self, ctx: &impl PathString) -> Result<GraphQLValue>;
}

impl ValueExt for DynamicValue {
    fn render_value<'a>(&self, ctx: &'a impl PathString) -> Result<GraphQLValue> {
        match self {
            DynamicValue::Value(value) => Ok(GraphQLValue::from_json(value.clone())?),
            DynamicValue::Mustache(m) => {
                let rendered: Cow<'a, str> = Cow::Owned(m.render(ctx));

                serde_json::from_str::<GraphQLValue>(rendered.as_ref())
                    // parsing can fail when Mustache::render returns bare string and since
                    // that string is not wrapped with quotes serde_json will fail to parse it
                    // but, we can just use that string as is
                    .or_else(|_| Ok(GraphQLValue::String(rendered.into_owned())))
            }
            DynamicValue::Object(obj) => {
                let out: Result<IndexMap<_, _>> = obj
                    .iter()
                    .map(|(k, v)| {
                        let key = Cow::Borrowed(k.as_str());
                        v.render_value(ctx).map(|val| (Name::new(&key), val))
                    })
                    .collect();
                out.map(GraphQLValue::Object)
            }
            DynamicValue::Array(arr) => {
                let out: Result<Vec<_>> = arr.iter().map(|v| v.render_value(ctx)).collect();
                out.map(GraphQLValue::List)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::blueprint::DynamicValue;
    use crate::serde_value_ext::ValueExt;

    #[test]
    fn test_render_value() {
        let value = json!({"a": "{{foo}}"});
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": "baz"}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": {"bar": "baz"}})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_nested() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": 1}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": 1})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_nested_str() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": "foo"}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": "foo"})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_nested_bool() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": true}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": true})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_nested_float() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": 1.1}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": 1.1})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_arr() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": [1,2,3]}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": [1, 2, 3]})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_render_value_arr_template() {
        let value = json!({"a": ["{{foo.bar.baz}}", "{{foo.bar.qux}}"]});
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": 1, "qux": 2}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": [1, 2]})).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_mustache_or_value_is_const() {
        let value = json!("{{foo}}");
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": "bar"});
        let result = value.render_value(&ctx).unwrap();
        let expected = async_graphql::Value::String("bar".to_owned());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mustache_arr_obj() {
        let value = json!([{"a": "{{foo.bar.baz}}"}, {"a": "{{foo.bar.qux}}"}]);
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": 1, "qux": 2}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!([{"a": 1}, {"a":2}])).unwrap();
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_mustache_arr_obj_arr() {
        let value = json!([{"a": [{"aa": "{{foo.bar.baz}}"}]}, {"a": [{"aa": "{{foo.bar.qux}}"}]}]);
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": 1, "qux": 2}}});
        let result = value.render_value(&ctx);
        let expected =
            async_graphql::Value::from_json(json!([{"a": [{"aa": 1}]}, {"a":[{"aa": 2}]}]))
                .unwrap();
        assert_eq!(result.unwrap(), expected);
    }
}
