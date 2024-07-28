use std::collections::HashMap;

use async_graphql::Name;
use async_graphql_value::ConstValue;
use indexmap::IndexMap;

use super::*;

impl<'a, Value: JsonLike<'a> + Clone> JsonObjectLike<'a> for IndexMap<Name, Value> {
    type Value = Value;

    fn new() -> Self {
        IndexMap::new()
    }

    fn get_key(&'a self, key: &str) -> Option<&Self::Value> {
        self.get(&Name::new(key))
    }

    fn insert_key(mut self, key: &'a str, value: Self::Value) -> Self {
        self.insert(Name::new(key), value);
        self
    }
}

impl<'a> JsonLike<'a> for ConstValue {
    type JsonObject = IndexMap<Name, ConstValue>;

    fn as_array(&'a self) -> Option<&'a Vec<Self>> {
        match self {
            ConstValue::List(seq) => Some(seq),
            _ => None,
        }
    }

    fn as_str(&'a self) -> Option<&'a str> {
        match self {
            ConstValue::String(s) => Some(s),
            _ => None,
        }
    }

    fn as_i64(&'a self) -> Option<i64> {
        match self {
            ConstValue::Number(n) => n.as_i64(),
            _ => None,
        }
    }

    fn as_u64(&'a self) -> Option<u64> {
        match self {
            ConstValue::Number(n) => n.as_u64(),
            _ => None,
        }
    }

    fn as_f64(&'a self) -> Option<f64> {
        match self {
            ConstValue::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    fn as_bool(&'a self) -> Option<bool> {
        match self {
            ConstValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    fn is_null(&'a self) -> bool {
        matches!(self, ConstValue::Null)
    }

    fn get_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&Self> {
        let mut val = self;
        for token in path {
            val = match val {
                ConstValue::List(seq) => {
                    let index = token.as_ref().parse::<usize>().ok()?;
                    seq.get(index)?
                }
                ConstValue::Object(map) => map.get(token.as_ref())?,
                _ => return None,
            };
        }
        Some(val)
    }

    fn get_key(&self, path: &str) -> Option<&Self> {
        match self {
            ConstValue::Object(map) => map.get(&async_graphql::Name::new(path)),
            _ => None,
        }
    }

    fn group_by(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }

    fn null() -> Self {
        Default::default()
    }

    fn as_object(&'a self) -> Option<&'a Self::JsonObject> {
        match self {
            ConstValue::Object(map) => Some(map),
            _ => None,
        }
    }

    fn object(obj: Self::JsonObject) -> Self {
        ConstValue::Object(obj)
    }

    fn array(arr: Vec<Self>) -> Self {
        ConstValue::List(arr)
    }

    fn string(s: &'a str) -> Self {
        ConstValue::String(s.to_string())
    }
}
