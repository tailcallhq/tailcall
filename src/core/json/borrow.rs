use super::{gather_path_matches, group_by_key, JsonLike, JsonObjectLike};
use serde_json_borrow::{ObjectAsVec, Value};

// Implementation for JsonObjectLike for ObjectAsVec
impl<'a> JsonObjectLike<'a> for ObjectAsVec<'a> {
    type Value = Value<'a>;

    fn get_key(&'a self, key: &str) -> Option<&Value<'a>> {
        self.get(key)
    }
}

// Implementation for JsonLike for Value
impl<'a> JsonLike<'a> for Value<'a> {
    type JsonObject = ObjectAsVec<'a>;

    fn null() -> Self {
        Value::Null
    }

    fn as_array(&'a self) -> Option<&'a Vec<Self>> {
        match self {
            Value::Array(array) => Some(array),
            _ => None,
        }
    }

    fn as_object(&'a self) -> Option<&Self::JsonObject> {
        self.as_object()
    }

    fn as_str(&'a self) -> Option<&str> {
        self.as_str()
    }

    fn as_i64(&'a self) -> Option<i64> {
        self.as_i64()
    }

    fn as_u64(&'a self) -> Option<u64> {
        self.as_u64()
    }

    fn as_f64(&'a self) -> Option<f64> {
        self.as_f64()
    }

    fn as_bool(&'a self) -> Option<bool> {
        self.as_bool()
    }

    fn is_null(&'a self) -> bool {
        self.is_null()
    }

    fn get_path<T: AsRef<str>>(&'a self, path: &'a [T]) -> Option<&Self> {
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

    fn get_key(&'a self, path: &'a str) -> Option<&Self> {
        match self {
            Value::Object(map) => map.get(path),
            _ => None,
        }
    }

    fn group_by(&'a self, path: &'a [String]) -> std::collections::HashMap<String, Vec<&'a Self>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }
}
