use std::borrow::Cow;
use std::collections::HashMap;

use serde_json::{Map, Value};

use super::{JsonLike, JsonObjectLike, JsonPrimitive};

impl<'obj> JsonObjectLike<'obj> for serde_json::Map<String, Value> {
    type Value = Value;

    fn new() -> Self {
        serde_json::Map::new()
    }

    fn with_capacity(n: usize) -> Self {
        serde_json::Map::with_capacity(n)
    }

    fn from_vec(v: Vec<(&'obj str, Self::Value)>) -> Self {
        Map::from_iter(v.into_iter().map(|(k, v)| (k.to_string(), v)))
    }

    fn get_key(&self, key: &str) -> Option<&Value> {
        self.get(key)
    }

    fn insert_key(&mut self, key: &'obj str, value: Self::Value) {
        self.insert(key.to_owned(), value);
    }

    fn iter<'slf>(&'slf self) -> impl Iterator<Item = (&'slf str, &'slf Self::Value)>
    where
        Self::Value: 'obj,
        'obj: 'slf,
    {
        self.iter().map(|(k, v)| (k.as_str(), v))
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<'json> JsonLike<'json> for Value {
    type JsonObject = serde_json::Map<String, Value>;

    fn from_primitive(x: JsonPrimitive<'json>) -> Self {
        match x {
            JsonPrimitive::Null => Value::Null,
            JsonPrimitive::Bool(x) => Value::Bool(x),
            JsonPrimitive::Str(s) => Value::String(s.to_string()),
            JsonPrimitive::Number(number) => Value::Number(number),
        }
    }

    fn as_primitive(&self) -> Option<JsonPrimitive> {
        let val = match self {
            Value::Null => JsonPrimitive::Null,
            Value::Bool(x) => JsonPrimitive::Bool(*x),
            Value::Number(number) => JsonPrimitive::Number(number.clone()),
            Value::String(s) => JsonPrimitive::Str(s.as_ref()),
            _ => return None,
        };

        Some(val)
    }

    fn as_array(&self) -> Option<&Vec<Self>> {
        self.as_array()
    }

    fn as_array_mut(&mut self) -> Option<&mut Vec<Self>> {
        self.as_array_mut()
    }

    fn into_array(self) -> Option<Vec<Self>> {
        if let Self::Array(vec) = self {
            Some(vec)
        } else {
            None
        }
    }

    fn as_str(&self) -> Option<&str> {
        self.as_str()
    }

    fn as_i64(&self) -> Option<i64> {
        self.as_i64()
    }

    fn as_u64(&self) -> Option<u64> {
        self.as_u64()
    }

    fn as_f64(&self) -> Option<f64> {
        self.as_f64()
    }

    fn as_bool(&self) -> Option<bool> {
        self.as_bool()
    }

    fn is_null(&self) -> bool {
        self.is_null()
    }

    fn get_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&Self> {
        let mut val = self;
        for token in path {
            val = match val {
                Value::Array(arr) => {
                    let index = token.as_ref().parse::<usize>().ok()?;
                    arr.get(index)?
                }
                Value::Object(map) => map.get(token.as_ref())?,
                _ => return None,
            };
        }
        Some(val)
    }

    fn get_key(&self, path: &str) -> Option<&Self> {
        match self {
            Value::Object(map) => map.get(path),
            _ => None,
        }
    }

    fn group_by(&self, path: &[String]) -> HashMap<String, Vec<&Self>> {
        let src = super::gather_path_matches(self, path, vec![]);
        super::group_by_key(src)
    }

    fn null() -> Self {
        Self::Null
    }

    fn as_object(&self) -> Option<&Self::JsonObject> {
        self.as_object()
    }

    fn as_object_mut(&mut self) -> Option<&mut Self::JsonObject> {
        self.as_object_mut()
    }

    fn into_object(self) -> Option<Self::JsonObject> {
        if let Self::Object(obj) = self {
            Some(obj)
        } else {
            None
        }
    }

    fn object(obj: Self::JsonObject) -> Self {
        Value::Object(obj)
    }

    fn array(arr: Vec<Self>) -> Self {
        Value::Array(arr)
    }

    fn string(s: Cow<'json, str>) -> Self {
        Value::String(s.to_string())
    }
}
