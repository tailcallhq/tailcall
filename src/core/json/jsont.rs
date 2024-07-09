#![allow(unused)]

use std::collections::HashMap;

pub trait JsonT {
    fn array_ok(value: &Self) -> Option<&Vec<Self>>
    where
        Self: Sized;
    fn str_ok(value: &Self) -> Option<&str>;
    fn i64_ok(value: &Self) -> Option<i64>;
    fn u64_ok(value: &Self) -> Option<u64>;
    fn f64_ok(value: &Self) -> Option<f64>;
    fn bool_ok(value: &Self) -> Option<bool>;
    fn null_ok(value: &Self) -> Option<()>;
    fn option_ok(value: &Self) -> Option<Option<&Self>>;
    fn get_path<'a, T: AsRef<str>>(value: &'a Self, path: &'a [T]) -> Option<&'a Self>;
    fn get_key<'a>(value: &'a Self, path: &'a str) -> Option<&'a Self>;
    fn new(value: &Self) -> &Self;
    fn group_by<'a>(value: &'a Self, path: &'a [String]) -> HashMap<String, Vec<&'a Self>>;
}

impl JsonT for serde_json::Value {
    fn array_ok(value: &Self) -> Option<&Vec<Self>> {
        value.as_array()
    }

    fn str_ok(value: &Self) -> Option<&str> {
        value.as_str()
    }

    fn i64_ok(value: &Self) -> Option<i64> {
        value.as_i64()
    }

    fn u64_ok(value: &Self) -> Option<u64> {
        value.as_u64()
    }

    fn f64_ok(value: &Self) -> Option<f64> {
        value.as_f64()
    }

    fn bool_ok(value: &Self) -> Option<bool> {
        value.as_bool()
    }

    fn null_ok(value: &Self) -> Option<()> {
        value.as_null()
    }

    fn option_ok(value: &Self) -> Option<Option<&Self>> {
        match value {
            serde_json::Value::Null => Some(None),
            value => Some(Some(value)),
        }
    }

    fn get_path<'a, T: AsRef<str>>(value: &'a Self, path: &'a [T]) -> Option<&'a Self> {
        let mut val = value;
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

    fn get_key<'a>(value: &'a Self, path: &'a str) -> Option<&'a Self> {
        value.get(path)
    }

    fn new(value: &Self) -> &Self {
        value
    }

    fn group_by<'a>(_value: &'a Self, _path: &'a [String]) -> HashMap<String, Vec<&'a Self>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::JsonT;

    #[test]
    fn test_array_ok() {
        let value = Value::Array(vec![Value::Null]);
        let result = <Value as JsonT>::array_ok(&value);

        assert_eq!(result, Some(&vec![Value::Null]));
    }
}
