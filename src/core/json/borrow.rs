use std::collections::HashMap;

use serde_json_borrow::{ObjectAsVec, Value};

use super::{gather_path_matches, group_by_key, JsonLike, JsonObjectLike};

// BorrowedValue
impl<'a> JsonObjectLike for ObjectAsVec<'a> {
    type Value<'json> = Value<'json> where 'a: 'json;

    fn new() -> Self {
        ObjectAsVec::default()
    }

    fn get_key<'b>(&'b self, key: &str) -> Option<&Self::Value<'b>> {
        self.get(key)
    }
}

impl<'ctx> JsonLike for Value<'ctx> {
    type JsonObject<'obj> = ObjectAsVec<'obj> where Self: 'obj;
    type Output<'a>  = Value<'a> where 'ctx: 'a, Self: 'a;

    fn null() -> Self {
        Value::Null
    }

    fn as_array(&self) -> Option<&Vec<Self>> {
        match self {
            Value::Array(array) => Some(array),
            _ => None,
        }
    }

    fn as_object(&self) -> Option<&Self::JsonObject<'_>> {
        self.as_object()
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Number(i) => i.as_i64(),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Number(i) => i.as_u64(),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(i) => i.as_f64(),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn is_null(&self) -> bool {
        self.is_null()
    }

    fn get_path<'a, T: AsRef<str>>(&'a self, path: &'a [T]) -> Option<&Self::Output<'a>> {
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

    fn get_key<'a>(&'a self, path: &'a str) -> Option<&Self::Output<'a>> {
        match self {
            Value::Object(map) => map.get(path),
            _ => None,
        }
    }

    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Output<'a>>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }
}
