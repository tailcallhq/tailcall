use std::borrow::Cow;
use std::collections::HashMap;

use super::{JsonLike, JsonObjectLike};

impl<'obj> JsonObjectLike<'obj> for serde_json::Map<String, serde_json::Value> {
    type Value = serde_json::Value;

    fn new() -> Self {
        serde_json::Map::new()
    }

    fn get_key(&self, key: &str) -> Option<&serde_json::Value> {
        self.get(key)
    }

    fn insert_key(&mut self, key: &'obj str, value: Self::Value) {
        self.insert(key.to_owned(), value);
    }
}

impl<'json> JsonLike<'json> for serde_json::Value {
    type JsonObject = serde_json::Map<String, serde_json::Value>;

    fn as_array(&self) -> Option<&Vec<Self>> {
        self.as_array()
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
                serde_json::Value::Array(arr) => {
                    let index = token.as_ref().parse::<usize>().ok()?;
                    arr.get(index)?
                }
                serde_json::Value::Object(map) => map.get(token.as_ref())?,
                _ => return None,
            };
        }
        Some(val)
    }

    fn get_key(&self, path: &str) -> Option<&Self> {
        match self {
            serde_json::Value::Object(map) => map.get(path),
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
        serde_json::Value::Object(obj)
    }

    fn array(arr: Vec<Self>) -> Self {
        serde_json::Value::Array(arr)
    }

    fn string(s: Cow<'json, str>) -> Self {
        serde_json::Value::String(s.to_string())
    }
}
