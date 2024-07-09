#![allow(unused)]

use std::collections::HashMap;

pub trait JsonT {
    type Output;
    type Input;

    fn array_ok(value: &Self::Input) -> Option<&Vec<Self::Output>>;
    fn str_ok(value: &Self::Input) -> Option<&str>;
    fn i64_ok(value: &Self::Input) -> Option<i64>;
    fn u64_ok(value: &Self::Input) -> Option<u64>;
    fn f64_ok(value: &Self::Input) -> Option<f64>;
    fn bool_ok(value: &Self::Input) -> Option<bool>;
    fn null_ok(value: &Self::Input) -> Option<()>;
    fn option_ok(value: &Self::Input) -> Option<Option<&Self::Output>>;
    fn get_path<'a, T: AsRef<str>>(value: &'a Self::Input, path: &'a [T]) -> Option<&'a Self::Output>;
    fn get_key<'a>(value: &'a Self::Input, path: &'a str) -> Option<&'a Self::Output>;
    fn new(value: &Self::Output) -> &Self;
    fn group_by<'a>(value: &'a Self::Input, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Output>>;
}

impl JsonT for serde_json::Value {
    type Output = serde_json::Value;
    type Input = serde_json::Value;

    fn array_ok(value: &Self::Input) -> Option<&Vec<Self::Output>> {
        value.as_array()
    }

    fn str_ok(value: &Self::Input) -> Option<&str> {
        value.as_str()
    }

    fn i64_ok(value: &Self::Input) -> Option<i64> {
        value.as_i64()
    }

    fn u64_ok(value: &Self::Input) -> Option<u64> {
        value.as_u64()
    }

    fn f64_ok(value: &Self::Input) -> Option<f64> {
        value.as_f64()
    }

    fn bool_ok(value: &Self::Input) -> Option<bool> {
        value.as_bool()
    }

    fn null_ok(value: &Self::Input) -> Option<()> {
        value.as_null()
    }

    fn option_ok(value: &Self::Input) -> Option<Option<&Self::Output>> {
        match value {
            serde_json::Value::Null => Some(None),
            value => Some(Some(value)),
        }
    }

    fn get_path<'a, T: AsRef<str>>(value: &'a Self::Input, path: &'a [T]) -> Option<&'a Self::Output> {
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

    fn get_key<'a>(value: &'a Self::Input, path: &'a str) -> Option<&'a Self::Output> {
        value.get(path)
    }

    fn new(value: &Self::Output) -> &Self {
        value
    }

    fn group_by<'a>(_value: &'a Self::Input, _path: &'a [String]) -> HashMap<String, Vec<&'a Self::Output>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::JsonT;
    use serde_json::Value;

    #[test]
    fn test_array_ok() {
        let value = Value::Array(vec![Value::Null]);
        let result = <Value as JsonT>::array_ok(&value);

        assert_eq!(result, Some(&vec![Value::Null]));
    }
}