#![allow(unused)]

use std::collections::HashMap;

use async_graphql_value::ConstValue;
use serde_json::Value;

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
    fn group_by<'a>(value: &'a Self, path: &'a [String]) -> HashMap<String, Vec<&'a Self>>;
}

impl JsonT for Value {
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
    fn array_ok(value: &Self) -> Option<&Vec<Self>>
    where
        Self: Sized,
    {
        match value {
            ConstValue::List(arr) => Some(arr),
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

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::core::json::group_by_key;
    use crate::core::json::json_like::gather_path_matches;

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
