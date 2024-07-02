use std::borrow::Cow;

use async_graphql::{Name, Value as GraphQLValue};
use indexmap::IndexMap;

use crate::core::blueprint::DynamicValue;
use crate::core::path::PathString;

pub trait ValueExt {
    fn render_value(&self, ctx: &impl PathString) -> GraphQLValue;
}

impl ValueExt for DynamicValue<async_graphql::Value> {
    fn render_value<'a>(&self, ctx: &'a impl PathString) -> GraphQLValue {
        match self {
            DynamicValue::Value(value) => value.to_owned(),
            DynamicValue::Mustache(m) => {
                let rendered: Cow<'a, str> = Cow::Owned(m.render(ctx));

                serde_json::from_str::<GraphQLValue>(rendered.as_ref())
                    // parsing can fail when Mustache::render returns bare string and since
                    // that string is not wrapped with quotes serde_json will fail to parse it
                    // but, we can just use that string as is
                    .unwrap_or_else(|_| GraphQLValue::String(rendered.into_owned()))
            }
            DynamicValue::Object(obj) => {
                let out: IndexMap<_, _> = obj
                    .iter()
                    .map(|(k, v)| {
                        let key = Cow::Borrowed(k.as_str());
                        let val = v.render_value(ctx);

                        (Name::new(key), val)
                    })
                    .collect();

                GraphQLValue::Object(out)
            }
            DynamicValue::Array(arr) => {
                let out: Vec<_> = arr.iter().map(|v| v.render_value(ctx)).collect();
                GraphQLValue::List(out)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::core::blueprint::DynamicValue;
    use crate::core::serde_value_ext::ValueExt;

    #[test]
    fn test_render_value() {
        let value = json!({"a": "{{foo}}"});
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": "baz"}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": {"bar": "baz"}})).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_render_value_nested() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": 1}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": 1})).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_render_value_nested_str() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": "foo"}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": "foo"})).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_render_value_null() {
        let value = json!("{{foo.bar.baz}}");
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": null}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!(null)).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_render_value_nested_bool() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": true}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": true})).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_render_value_nested_float() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": 1.1}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": 1.1})).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_render_value_arr() {
        let value = json!({"a": "{{foo.bar.baz}}"});
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": [1,2,3]}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": [1, 2, 3]})).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_render_value_arr_template() {
        let value = json!({"a": ["{{foo.bar.baz}}", "{{foo.bar.qux}}"]});
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": {"bar": {"baz": 1, "qux": 2}}});
        let result = value.render_value(&ctx);
        let expected = async_graphql::Value::from_json(json!({"a": [1, 2]})).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mustache_or_value_is_const() {
        let value = json!("{{foo}}");
        let value = DynamicValue::try_from(&value).unwrap();
        let ctx = json!({"foo": "bar"});
        let result = value.render_value(&ctx);
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
        assert_eq!(result, expected);
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
        assert_eq!(result, expected);
    }
}
