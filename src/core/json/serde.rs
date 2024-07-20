use std::collections::HashMap;

use super::{JsonLike, JsonObjectLike};

impl JsonObjectLike for serde_json::Map<String, serde_json::Value> {
    type Value<'a> = serde_json::Value where Self: 'a;
    type Object<'obj> = serde_json::Map<String, serde_json::Value> where Self: 'obj;

    fn new() -> Self {
        serde_json::Map::new()
    }

    fn get_key(&self, key: &str) -> Option<&serde_json::Value> {
        self.get(key)
    }

    fn insert_key<'a>(
        mut slf: Self::Object<'a>,
        key: &'a str,
        value: Self::Value<'a>,
    ) -> Self::Object<'a>
    where
        Self: 'a,
    {
        slf.insert(key.to_string(), value);
        slf
    }
}

impl JsonLike for serde_json::Value {
    type JsonObject = serde_json::Map<String, serde_json::Value>;
    type Output<'a> = serde_json::Value where Self: 'a;

    fn null() -> Self {
        Self::Null
    }

    fn as_array(&self) -> Option<&Vec<Self>> {
        self.as_array()
    }

    fn as_object(&self) -> Option<&Self::JsonObject> {
        self.as_object()
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
    fn get_path<'a, T: AsRef<str>>(&'a self, path: &'a [T]) -> Option<&Self::Output<'a>> {
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

    fn get_key<'a>(&'a self, path: &'a str) -> Option<&Self::Output<'a>> {
        match self {
            serde_json::Value::Object(map) => map.get(path),
            _ => None,
        }
    }

    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&Self::Output<'a>>> {
        let src = super::gather_path_matches(self, path, vec![]);
        super::group_by_key(src)
    }
}
