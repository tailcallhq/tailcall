use std::borrow::Cow;

use async_graphql_value::ConstValue;

pub fn to_borrowed(value: &ConstValue) -> serde_json_borrow::Value<'_> {
    match value {
        ConstValue::Null => serde_json_borrow::Value::Null,
        ConstValue::Boolean(b) => serde_json_borrow::Value::Bool(*b),
        ConstValue::Number(n) => {
            if n.is_i64() {
                serde_json_borrow::Value::Number(n.as_i64().unwrap().into())
            } else if n.is_u64() {
                serde_json_borrow::Value::Number(n.as_u64().unwrap().into())
            } else {
                serde_json_borrow::Value::Number(n.as_f64().unwrap().into())
            }
        }
        ConstValue::String(s) => serde_json_borrow::Value::Str(Cow::Borrowed(s)),
        ConstValue::List(l) => serde_json_borrow::Value::Array(l.iter().map(to_borrowed).collect()),
        ConstValue::Object(o) => serde_json_borrow::Value::Object(
            o.iter()
                .map(|(k, v)| (k.as_str(), to_borrowed(v)))
                .collect::<Vec<_>>()
                .into(),
        ),
        _ => serde_json_borrow::Value::Null, // TODO: impl for rest of the types
    }
}
