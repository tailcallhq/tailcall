use std::collections::HashMap;

use super::{JsonArrayLike, JsonLike, JsonObjectLike};

// SerdeValue
impl<'a> JsonObjectLike<'a> for serde_json::Map<String, serde_json::Value> {
    type Value = serde_json::Value;
    fn get_key(&'a self, key: &str) -> Option<&serde_json::Value> {
        self.get(key)
    }
}

impl<'a> JsonArrayLike<'a> for Vec<serde_json::Value> {
    type Value = serde_json::Value;
    fn as_vec(&'a self) -> &'a Vec<serde_json::Value> {
        self
    }
}

impl<'a> JsonLike<'a> for serde_json::Value {
    type JsonObject = serde_json::Map<String, serde_json::Value>;
    type JsonArray = Vec<serde_json::Value>;

    fn as_array_ok(&self) -> Result<&Self::JsonArray, &str> {
        self.as_array().ok_or("expected array")
    }
    fn as_str_ok(&self) -> Result<&str, &str> {
        self.as_str().ok_or("expected str")
    }
    fn as_i64_ok(&self) -> Result<i64, &str> {
        self.as_i64().ok_or("expected i64")
    }
    fn as_u64_ok(&self) -> Result<u64, &str> {
        self.as_u64().ok_or("expected u64")
    }
    fn as_f64_ok(&self) -> Result<f64, &str> {
        self.as_f64().ok_or("expected f64")
    }
    fn as_bool_ok(&self) -> Result<bool, &str> {
        self.as_bool().ok_or("expected bool")
    }
    fn as_null_ok(&self) -> Result<(), &str> {
        self.as_null().ok_or("expected null")
    }

    fn as_option_ok(&self) -> Result<Option<&Self>, &str> {
        match self {
            serde_json::Value::Null => Ok(None),
            _ => Ok(Some(self)),
        }
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

    fn group_by(&'a self, path: &'a [String]) -> HashMap<String, Vec<&Self>> {
        let src = super::gather_path_matches(self, path, vec![]);
        super::group_by_key(src)
    }

    fn null() -> Self {
        Self::Null
    }

    fn as_object_ok(&self) -> Result<&Self::JsonObject, &str> {
        match self {
            serde_json::Value::Object(map) => Ok(map),
            _ => Err("expected object"),
        }
    }
}
