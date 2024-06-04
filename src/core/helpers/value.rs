use std::hash::{Hash, Hasher};

use async_graphql::Name;
use async_graphql_value::ConstValue;

#[derive(Clone, Eq)]
pub struct HashableConstValue(pub ConstValue);

impl Hash for HashableConstValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash(&self.0, state)
    }
}

impl PartialEq for HashableConstValue {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

pub fn hash<H: Hasher>(const_value: &ConstValue, state: &mut H) {
    match const_value {
        ConstValue::Null => {}
        ConstValue::Boolean(val) => val.hash(state),
        ConstValue::Enum(name) => name.hash(state),
        ConstValue::Number(num) => num.hash(state),
        ConstValue::Binary(bytes) => bytes.hash(state),
        ConstValue::String(string) => string.hash(state),
        ConstValue::List(list) => list.iter().for_each(|val| hash(val, state)),
        ConstValue::Object(object) => {
            let mut tmp_list: Vec<_> = object.iter().collect();
            tmp_list.sort_by(|(key1, _), (key2, _)| key1.cmp(key2));
            tmp_list.iter().for_each(|(key, value)| {
                key.hash(state);
                hash(value, state);
            })
        }
    }
}

pub fn from_serde_owned(value: serde_json_borrow::Value) -> ConstValue {
    use serde_json_borrow::Value;
    match value {
        Value::Null => ConstValue::Null,
        Value::Bool(b) => ConstValue::Boolean(b),
        Value::Number(n) => ConstValue::Number(n.into()),
        Value::Str(s) => ConstValue::String(s.into()),
        Value::Array(a) => ConstValue::List(a.into_iter().map(from_serde_owned).collect()),
        Value::Object(o) => ConstValue::Object(
            o.iter()
                .map(|(k, v)| (Name::new(k), from_serde_owned(v.to_owned())))
                .collect(),
        ),
    }
}
