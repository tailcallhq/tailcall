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

    fn group_by<'a>(value: &'a Self, path: &'a [String]) -> HashMap<String, Vec<&'a Self>> {
        let src = gather_path_matches(value, path, vec![]);
        group_by_key(src)
    }
}

// Highly micro-optimized and benchmarked version of get_path_all
// Any further changes should be verified with benchmarks
pub fn gather_path_matches<'a, J: JsonT>(
    root: &'a J,
    path: &'a [String],
    mut vector: Vec<(&'a J, &'a J)>,
) -> Vec<(&'a J, &'a J)> {
    if let Some(root) = <J as JsonT>::array_ok(root) {
        for value in root {
            vector = gather_path_matches(J::new(value), path, vector);
        }
    } else if let Some((key, tail)) = path.split_first() {
        if let Some(value) = <J as JsonT>::get_key(root, key) {
            if tail.is_empty() {
                vector.push((J::new(value), root));
            } else {
                vector = gather_path_matches(J::new(value), tail, vector);
            }
        }
    }

    vector
}

pub fn group_by_key<'a, J: JsonT>(src: Vec<(&'a J, &'a J)>) -> HashMap<String, Vec<&'a J>> {
    let mut map: HashMap<String, Vec<&'a J>> = HashMap::new();
    for (key, value) in src {
        // Need to handle number and string keys
        let key_str = <J as JsonT>::str_ok(key)
            .map(|v| v.to_string())
            .or_else(|| <J as JsonT>::i64_ok(key).map(|v| v.to_string()));

        if let Some(key) = key_str {
            if let Some(values) = map.get_mut(&key) {
                values.push(value);
            } else {
                map.insert(key, vec![value]);
            }
        }
    }
    map
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
