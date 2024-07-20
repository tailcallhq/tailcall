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


    // fn insert_key(&'a mut self, key: &'a str, value: Self::Value) {
    //     self.insert(key, value);
    // }
}

impl<'ctx> JsonLike for Value<'ctx> {
    type JsonObject<'obj> = ObjectAsVec<'obj> where Self: 'obj;
    type Output<'a>  = Value<'a> where 'ctx: 'a, Self: 'a;

    fn null() -> Self {
        Value::Null
    }

   /* fn own<'a>(value: &'a Self::Output<'a>) -> &'a Self {
        value
    }*/

    fn as_array<'a>(&'a self) -> Option<&'a Vec<Self>> {
        match self {
            Value::Array(array) => Some(array),
            _ => None,
        }
    }

    fn as_object<'a>(&'a self) -> Option<&Self::JsonObject<'a>> {
        self.as_object()
    }

    fn as_str<'a>(&'a self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    fn as_i64<'a>(&'a self) -> Option<i64> {
        match self {
            Value::Number(i) => i.as_i64(),
            _ => None,
        }
    }

    fn as_u64<'a>(&'a self) -> Option<u64> {
        match self {
            Value::Number(i) => i.as_u64(),
            _ => None,
        }
    }

    fn as_f64<'a>(&'a self) -> Option<f64> {
        match self {
            Value::Number(i) => i.as_f64(),
            _ => None,
        }
    }

    fn as_bool<'a>(&'a self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn is_null<'a>(&'a self) -> bool {
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
    /*    type JsonObject<'obj> = ObjectAsVec<'obj> where 'a: 'obj;

        fn null() -> Self {
            Value::Null
        }

        fn as_array(&self) -> Option<&Vec<Value>> {
            match self {
                Value::Array(array) => Some(array),
                _ => None,
            }
        }

        fn as_object(&'a self) -> Option<&Self::JsonObject<'a>> {
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

        fn get_key(&'a self, _path: &'a str) -> Option<&Self> {
            match self {
                Value::Object(map) => map.get(_path),
                _ => None,
            }
        }

        fn group_by(&'a self, path: &'a [String]) -> std::collections::HashMap<String, Vec<&Self>> {
            let src = gather_path_matches(self, path, vec![]);
            group_by_key(src)
        }*/
}
