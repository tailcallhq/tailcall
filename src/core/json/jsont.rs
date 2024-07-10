#![allow(unused)]

use std::collections::HashMap;

use async_graphql_value::ConstValue;
use serde_json::Value;
use serde_json_borrow::{ObjectAsVec, Value as BorrowedValue};

use crate::core::json::JsonObjectLike;

/// A trait for JSON-like objects
/// This trait is used to abstract over different JSON-like objects
pub trait JsonT {
    type JsonObject: JsonObjectLike;

    // Constructors

    /// Create a default value
    fn default() -> Self;

    /// Create a new array
    fn new_array(arr: Vec<Self>) -> Self
    where
        Self: Sized;

    /// Create a new value
    fn new(value: &Self) -> &Self;

    // Operators

    /// Get the array if the value is an array
    fn array_ok(value: &Self) -> Option<&[Self]>
    where
        Self: Sized;

    /// Get the object if the value is an object
    fn object_ok(value: &Self) -> Option<&Self::JsonObject>;

    /// Get the string if the value is a string
    fn str_ok(value: &Self) -> Option<&str>;

    /// Get the i64 if the value is an i64
    fn i64_ok(value: &Self) -> Option<i64>;

    /// Get the u64 if the value is a u64
    fn u64_ok(value: &Self) -> Option<u64>;

    /// Get the f64 if the value is an f64
    fn f64_ok(value: &Self) -> Option<f64>;

    /// Get the bool if the value is a bool
    fn bool_ok(value: &Self) -> Option<bool>;

    /// Get the null if the value is a null
    fn null_ok(value: &Self) -> Option<()>;

    /// Get the option if the value is a null
    fn option_ok(value: &Self) -> Option<Option<&Self>>;

    /// Get the value at a path
    fn get_path<'a, T: AsRef<str>>(value: &'a Self, path: &'a [T]) -> Option<&'a Self>;

    /// Get the value at a key
    fn get_key<'a>(value: &'a Self, path: &'a str) -> Option<&'a Self>;

    /// Group the values by a path
    fn group_by<'a>(value: &'a Self, path: &'a [String]) -> HashMap<String, Vec<&'a Self>>;
}

impl JsonT for Value {
    type JsonObject = serde_json::Map<String, Value>;

    fn default() -> Self {
        Default::default()
    }

    fn new_array(arr: Vec<Self>) -> Self {
        Value::Array(arr)
    }

    fn new(value: &Self) -> &Self {
        value
    }

    fn array_ok(value: &Self) -> Option<&[Self]> {
        match value {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }

    fn object_ok(value: &Self) -> Option<&Self::JsonObject> {
        match value {
            Value::Object(map) => Some(map),
            _ => None,
        }
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
            vector = gather_path_matches(value, path, vector);
        }
    } else if let Some((key, tail)) = path.split_first() {
        if let Some(value) = <J as JsonT>::get_key(root, key) {
            if tail.is_empty() {
                vector.push((value, root));
            } else {
                vector = gather_path_matches(value, tail, vector);
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

impl JsonT for async_graphql::Value {
    type JsonObject = indexmap::IndexMap<async_graphql::Name, async_graphql::Value>;

    fn default() -> Self {
        Default::default()
    }

    fn new_array(arr: Vec<Self>) -> Self {
        ConstValue::List(arr)
    }

    fn new(value: &Self) -> &Self {
        value
    }

    fn array_ok(value: &Self) -> Option<&[Self]>
    where
        Self: Sized,
    {
        match value {
            ConstValue::List(arr) => Some(arr),
            _ => None,
        }
    }

    fn object_ok(value: &Self) -> Option<&Self::JsonObject> {
        match value {
            ConstValue::Object(map) => Some(map),
            _ => None,
        }
    }

    fn str_ok(value: &Self) -> Option<&str> {
        match value {
            ConstValue::String(s) => Some(s),
            _ => None,
        }
    }

    fn i64_ok(value: &Self) -> Option<i64> {
        match value {
            ConstValue::Number(n) => n.as_i64(),
            _ => None,
        }
    }

    fn u64_ok(value: &Self) -> Option<u64> {
        match value {
            ConstValue::Number(n) => n.as_u64(),
            _ => None,
        }
    }

    fn f64_ok(value: &Self) -> Option<f64> {
        match value {
            ConstValue::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    fn bool_ok(value: &Self) -> Option<bool> {
        match value {
            ConstValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    fn null_ok(value: &Self) -> Option<()> {
        match value {
            ConstValue::Null => Some(()),
            _ => None,
        }
    }

    fn option_ok(value: &Self) -> Option<Option<&Self>> {
        match value {
            ConstValue::Null => Some(None),
            _ => Some(Some(value)),
        }
    }

    fn get_path<'a, T: AsRef<str>>(value: &'a Self, path: &'a [T]) -> Option<&'a Self> {
        let mut val = value;
        for token in path {
            val = match val {
                ConstValue::List(arr) => {
                    let index = token.as_ref().parse::<usize>().ok()?;
                    arr.get(index)?
                }
                ConstValue::Object(map) => map.get(token.as_ref())?,
                _ => return None,
            };
        }
        Some(val)
    }

    fn get_key<'a>(value: &'a Self, path: &'a str) -> Option<&'a Self> {
        match value {
            ConstValue::Object(map) => map.get(path),
            _ => None,
        }
    }

    fn group_by<'a>(value: &'a Self, path: &'a [String]) -> HashMap<String, Vec<&'a Self>> {
        let src = gather_path_matches(value, path, vec![]);
        group_by_key(src)
    }
}

impl<'ctx> JsonT for BorrowedValue<'ctx> {
    type JsonObject = ObjectAsVec<'ctx>;

    fn default() -> Self {
        BorrowedValue::Null
    }

    fn new_array(arr: Vec<Self>) -> Self
    where
        Self: Sized,
    {
        BorrowedValue::Array(arr)
    }

    fn new(value: &Self) -> &Self {
        value
    }

    fn array_ok(value: &Self) -> Option<&[Self]>
    where
        Self: Sized,
    {
        match value {
            BorrowedValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    fn object_ok(value: &Self) -> Option<&Self::JsonObject> {
        match value {
            BorrowedValue::Object(map) => Some(map),
            _ => None,
        }
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
        match value {
            BorrowedValue::Null => Some(()),
            _ => None,
        }
    }

    fn option_ok(value: &Self) -> Option<Option<&Self>> {
        match value {
            BorrowedValue::Null => Some(None),
            value => Some(Some(value)),
        }
    }

    fn get_path<'a, T: AsRef<str>>(value: &'a Self, path: &'a [T]) -> Option<&'a Self> {
        todo!()
    }

    fn get_key<'a>(value: &'a Self, path: &'a str) -> Option<&'a Self> {
        todo!()
    }

    fn group_by<'a>(value: &'a Self, path: &'a [String]) -> HashMap<String, Vec<&'a Self>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_gather_path_matches() {
        let input = json!([
            {"id": "1"},
            {"id": "2"},
            {"id": "3"}
        ]);

        let actual =
            serde_json::to_value(gather_path_matches(&input, &["id".into()], vec![])).unwrap();

        let expected = json!(
            [
              ["1", {"id": "1"}],
              ["2", {"id": "2"}],
              ["3", {"id": "3"}],
            ]
        );

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_gather_path_matches_nested() {
        let input = json!({
            "data": [
                {"user": {"id": "1"}},
                {"user": {"id": "2"}},
                {"user": {"id": "3"}},
                {"user": [
                    {"id": "4"},
                    {"id": "5"}
                    ]
                },
            ]
        });

        let actual = serde_json::to_value(gather_path_matches(
            &input,
            &["data".into(), "user".into(), "id".into()],
            vec![],
        ))
        .unwrap();

        let expected = json!(
            [
              ["1", {"id": "1"}],
              ["2", {"id": "2"}],
              ["3", {"id": "3"}],
              ["4", {"id": "4"}],
              ["5", {"id": "5"}],

            ]
        );

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_group_by_key() {
        let arr = vec![
            (json!("1"), json!({"id": "1"})),
            (json!("2"), json!({"id": "2"})),
            (json!("2"), json!({"id": "2"})),
            (json!("3"), json!({"id": "3"})),
        ];
        let input: Vec<(&serde_json::Value, &serde_json::Value)> =
            arr.iter().map(|a| (&a.0, &a.1)).collect();

        let actual = serde_json::to_value(group_by_key(input)).unwrap();

        let expected = json!(
            {
                "1": [{"id": "1"}],
                "2": [{"id": "2"}, {"id": "2"}],
                "3": [{"id": "3"}],
            }
        );

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_group_by_numeric_key() {
        let arr = vec![
            (json!(1), json!({"id": 1})),
            (json!(2), json!({"id": 2})),
            (json!(2), json!({"id": 2})),
            (json!(3), json!({"id": 3})),
        ];
        let input: Vec<(&serde_json::Value, &serde_json::Value)> =
            arr.iter().map(|a| (&a.0, &a.1)).collect();

        let actual = serde_json::to_value(group_by_key(input)).unwrap();

        let expected = json!(
            {
                "1": [{"id": 1}],
                "2": [{"id": 2}, {"id": 2}],
                "3": [{"id": 3}],
            }
        );

        assert_eq!(actual, expected)
    }
}
