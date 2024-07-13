use serde_json_borrow::{ObjectAsVec, Value};

use super::{gather_path_matches, group_by_key, JsonArrayLike, JsonLike, JsonObjectLike};

// BorrowedValue
impl<'a> JsonObjectLike<'a> for ObjectAsVec<'a> {
    type Value = Value<'a>;
    fn get_key(&'a self, key: &str) -> Option<&Value> {
        self.get(key)
    }
}

impl<'a> JsonArrayLike<'a> for Vec<Value<'a>> {
    type Value = Value<'a>;
    fn as_vec(&'a self) -> &'a Vec<Value> {
        self
    }
}

impl<'a> JsonLike<'a> for Value<'a> {
    type JsonObject = ObjectAsVec<'a>;
    type JsonArray = Vec<Value<'a>>;

    fn null() -> Self {
        Value::Null
    }

    fn as_array_ok(&'a self) -> Result<&Self::JsonArray, &str> {
        match self {
            Value::Array(array) => Ok(array),
            _ => Err("expected array"),
        }
    }

    fn as_object_ok(&'a self) -> Result<&Self::JsonObject, &str> {
        self.as_object().ok_or("expected object")
    }

    fn as_str_ok(&'a self) -> Result<&str, &str> {
        self.as_str().ok_or("expected str")
    }

    fn as_i64_ok(&'a self) -> Result<i64, &str> {
        self.as_i64().ok_or("expected i64")
    }

    fn as_u64_ok(&'a self) -> Result<u64, &str> {
        self.as_u64().ok_or("expected u64")
    }

    fn as_f64_ok(&'a self) -> Result<f64, &str> {
        self.as_f64().ok_or("expected f64")
    }

    fn as_bool_ok(&'a self) -> Result<bool, &str> {
        self.as_bool().ok_or("expected bool")
    }

    fn as_null_ok(&'a self) -> Result<(), &str> {
        if self.is_null() {
            Ok(())
        } else {
            Err("expected null")
        }
    }

    fn as_option_ok(&'a self) -> Result<Option<&Self>, &str> {
        match self {
            Value::Null => Ok(None),
            _ => Ok(Some(self)),
        }
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

    fn get_key(&'a self, _path: &'a str) -> Option<&Self> {
        match self {
            Value::Object(map) => map.get(_path),
            _ => None,
        }
    }

    fn group_by(&'a self, path: &'a [String]) -> std::collections::HashMap<String, Vec<&Self>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }
}
