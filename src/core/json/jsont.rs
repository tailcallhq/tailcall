#![allow(unused)]

use std::collections::HashMap;

pub trait JsonT {
    type Output;
    type Input;

    fn array_ok(value: &Self::Input) -> Result<&Vec<Self::Output>, &str>;
    fn str_ok(value: &Self::Input) -> Result<&str, &str>;
    fn i64_ok(value: &Self::Input) -> Result<i64, &str>;
    fn u64_ok(value: &Self::Input) -> Result<u64, &str>;
    fn f64_ok(value: &Self::Input) -> Result<f64, &str>;
    fn bool_ok(value: &Self::Input) -> Result<bool, &str>;
    fn null_ok(value: &Self::Input) -> Result<(), &str>;
    fn option_ok(value: &Self::Input) -> Result<Option<&Self::Output>, &str>;
    fn get_path<'a, T: AsRef<str>>(value: &'a Self::Input, path: &'a [T]) -> Option<&'a Self::Output>;
    fn get_key<'a>(value: &'a Self::Input, path: &'a str) -> Option<&'a Self::Output>;
    fn new(value: &Self::Output) -> &Self;
    fn group_by<'a>(value: &'a Self::Input, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Output>>;
}

impl JsonT for serde_json::Value {
    type Output = serde_json::Value;
    type Input = serde_json::Value;

    fn array_ok(value: &Self::Input) -> Result<&Vec<Self::Output>, &str> {
        value.as_array().ok_or("expected array")
    }

    fn str_ok(value: &Self::Input) -> Result<&str, &str> {
        value.as_str().ok_or("expected str")
    }

    fn i64_ok(value: &Self::Input) -> Result<i64, &str> {
        value.as_i64().ok_or("expected i64")
    }

    fn u64_ok(value: &Self::Input) -> Result<u64, &str> {
        value.as_u64().ok_or("expected u64")
    }

    fn f64_ok(value: &Self::Input) -> Result<f64, &str> {
        value.as_f64().ok_or("expected f64")
    }

    fn bool_ok(value: &Self::Input) -> Result<bool, &str> {
        value.as_bool().ok_or("expected bool")
    }

    fn null_ok(value: &Self::Input) -> Result<(), &str> {
        value.as_null().ok_or("expected null")
    }

    fn option_ok(value: &Self::Input) -> Result<Option<&Self::Output>, &str> {
        match value {
            serde_json::Value::Null => Ok(None),
            _ => Ok(Some(value)),
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

        assert_eq!(result, Ok(&vec![Value::Null]));
    }
}