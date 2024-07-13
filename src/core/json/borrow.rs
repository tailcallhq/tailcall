use serde_json_borrow::{ObjectAsVec, Value};

use super::{JsonArrayLike, JsonLike, JsonObjectLike};

impl<'a> JsonLike for Value<'a> {
    type JsonObject<'i> = ObjectAsVec<'i> where 'a: 'i;
    type JsonArray<'i> = Vec<Value<'i>> where 'a: 'i;

    fn null() -> Self {
        Value::Null
    }

    fn as_array_ok<'b>(&'b self) -> Result<&Self::JsonArray<'b>, &str> {
        match self {
            Value::Array(array) => Ok(array),
            _ => Err("expected array"),
        }
    }

    fn as_object_ok<'b>(&'b self) -> Result<&Self::JsonObject<'b>, &str> {
        todo!()
    }

    fn as_str_ok(&self) -> Result<&str, &str> {
        todo!()
    }

    fn as_i64_ok(&self) -> Result<i64, &str> {
        todo!()
    }

    fn as_u64_ok(&self) -> Result<u64, &str> {
        todo!()
    }

    fn as_f64_ok(&self) -> Result<f64, &str> {
        todo!()
    }

    fn as_bool_ok(&self) -> Result<bool, &str> {
        todo!()
    }

    fn as_null_ok(&self) -> Result<(), &str> {
        todo!()
    }

    fn as_option_ok(&self) -> Result<Option<&Self>, &str> {
        todo!()
    }

    fn get_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&Self> {
        todo!()
    }

    fn get_key(&self, path: &str) -> Option<&Self> {
        todo!()
    }

    fn group_by<'b>(
        &'b self,
        path: &'b [String],
    ) -> std::collections::HashMap<String, Vec<&'b Self>> {
        todo!()
    }
}

// BorrowedValue
impl<'a> JsonObjectLike for serde_json_borrow::ObjectAsVec<'a> {
    type Value<'i> = serde_json_borrow::Value<'i> where 'a: 'i;

    fn get_key<'b>(&'b self, key: &str) -> Option<&Self::Value<'b>> {
        self.get(key)
    }
}

impl<'i> JsonArrayLike for std::vec::Vec<serde_json_borrow::Value<'i>> {
    type Value<'a> = serde_json_borrow::Value<'a>
    where
        Self: 'a;

    fn as_vec<'a>(&'a self) -> &'a Vec<&Self::Value<'a>> {
        todo!()
    }
}
