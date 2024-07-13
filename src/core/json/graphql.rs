use super::{gather_path_matches, group_by_key, JsonLike, JsonObjectLike};
use async_graphql::Name;
use async_graphql_value::ConstValue;
use indexmap::IndexMap;
use std::collections::HashMap;

// Implementation for JsonObjectLike for IndexMap<Name, Value>
impl<'a, Value: JsonLike<'a> + Clone> JsonObjectLike<'a> for IndexMap<Name, Value> {
    type Value = Value;

    fn get_key(&'a self, key: &str) -> Option<&Self::Value> {
        self.get(&Name::new(key))
    }
}

// Implementation for JsonLike for ConstValue
impl<'a> JsonLike<'a> for ConstValue {
    type JsonObject = IndexMap<Name, ConstValue>;

    fn null() -> Self {
        ConstValue::Null
    }

    fn as_array(&'a self) -> Option<&'a Vec<Self>> {
        match self {
            ConstValue::List(seq) => Some(seq),
            _ => None,
        }
    }

    fn as_object(&'a self) -> Option<&Self::JsonObject> {
        match self {
            ConstValue::Object(map) => Some(map),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            ConstValue::String(s) => Some(s),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            ConstValue::Number(n) => n.as_i64(),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match self {
            ConstValue::Number(n) => n.as_u64(),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            ConstValue::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            ConstValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, ConstValue::Null)
    }

    fn get_path<T: AsRef<str>>(&'a self, path: &'a [T]) -> Option<&Self> {
        let mut val = self;
        for token in path {
            val = match val {
                ConstValue::List(seq) => {
                    let index = token.as_ref().parse::<usize>().ok()?;
                    seq.get(index)?
                }
                ConstValue::Object(map) => map.get(&Name::new(token.as_ref()))?,
                _ => return None,
            };
        }
        Some(val)
    }

    fn get_key(&'a self, path: &'a str) -> Option<&Self> {
        match self {
            ConstValue::Object(map) => map.get(&Name::new(path)),
            _ => None,
        }
    }

    fn group_by(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }
}
