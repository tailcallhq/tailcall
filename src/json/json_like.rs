use std::collections::HashMap;

use async_graphql_value::ConstValue;

pub trait JsonLike {
    type Output;
    fn as_array_ok(&self) -> Result<&Vec<Self::Output>, &str>;
    fn as_str_ok(&self) -> Result<&str, &str>;
    fn as_i64_ok(&self) -> Result<i64, &str>;
    fn as_u64_ok(&self) -> Result<u64, &str>;
    fn as_f64_ok(&self) -> Result<f64, &str>;
    fn as_bool_ok(&self) -> Result<bool, &str>;
    fn as_null_ok(&self) -> Result<(), &str>;
    fn as_option_ok(&self) -> Result<Option<&Self::Output>, &str>;
    fn get_path(&self, path: &[String]) -> Option<&Self::Output>;
    fn new(value: Self::Output) -> Self;
}

impl JsonLike for serde_json::Value {
    type Output = serde_json::Value;
    fn as_array_ok(&self) -> Result<&Vec<Self::Output>, &str> {
        self.as_array().ok_or("expected array")
    }
    fn as_str_ok(&self) -> Result<&str, &str> {
        self.as_str().ok_or("expected str")
    }
    fn as_i64_ok(&self) -> Result<i64, &str> {
        self.as_i64().ok_or("expected i64")
    }
    fn as_u64_ok(&self) -> Result<u64, &str> {
        self.as_u64().ok_or("expected u64")
    }
    fn as_f64_ok(&self) -> Result<f64, &str> {
        self.as_f64().ok_or("expected f64")
    }
    fn as_bool_ok(&self) -> Result<bool, &str> {
        self.as_bool().ok_or("expected bool")
    }
    fn as_null_ok(&self) -> Result<(), &str> {
        self.as_null().ok_or("expected null")
    }

    fn as_option_ok(&self) -> Result<Option<&Self::Output>, &str> {
        match self {
            serde_json::Value::Null => Ok(None),
            _ => Ok(Some(self)),
        }
    }

    fn get_path(&self, path: &[String]) -> Option<&Self::Output> {
        let mut val = self;
        for token in path {
            val = match val {
                serde_json::Value::Array(arr) => {
                    let index = token.parse::<usize>().ok()?;
                    arr.get(index)?
                }
                serde_json::Value::Object(map) => map.get(token)?,
                _ => return None,
            };
        }
        Some(val)
    }

    fn new(value: Self::Output) -> Self {
        value
    }
}

impl JsonLike for async_graphql::Value {
    type Output = async_graphql::Value;

    fn as_array_ok(&self) -> Result<&Vec<Self::Output>, &str> {
        match self {
            ConstValue::List(seq) => Ok(seq),
            _ => Err("array"),
        }
    }

    fn as_str_ok(&self) -> Result<&str, &str> {
        match self {
            ConstValue::String(s) => Ok(s),
            _ => Err("str"),
        }
    }

    fn as_i64_ok(&self) -> Result<i64, &str> {
        match self {
            ConstValue::Number(n) => n.as_i64().ok_or("expected i64"),
            _ => Err("i64"),
        }
    }

    fn as_u64_ok(&self) -> Result<u64, &str> {
        match self {
            ConstValue::Number(n) => n.as_u64().ok_or("expected u64"),
            _ => Err("u64"),
        }
    }

    fn as_f64_ok(&self) -> Result<f64, &str> {
        match self {
            ConstValue::Number(n) => n.as_f64().ok_or("expected f64"),
            _ => Err("f64"),
        }
    }

    fn as_bool_ok(&self) -> Result<bool, &str> {
        match self {
            ConstValue::Boolean(b) => Ok(*b),
            _ => Err("bool"),
        }
    }

    fn as_null_ok(&self) -> Result<(), &str> {
        match self {
            ConstValue::Null => Ok(()),
            _ => Err("null"),
        }
    }

    fn as_option_ok(&self) -> Result<Option<&Self::Output>, &str> {
        match self {
            ConstValue::Null => Ok(None),
            _ => Ok(Some(self)),
        }
    }

    fn get_path(&self, path: &[String]) -> Option<&Self::Output> {
        let mut val = self;
        for token in path {
            val = match val {
                ConstValue::List(seq) => {
                    let index = token.parse::<usize>().ok()?;
                    seq.get(index)?
                }
                ConstValue::Object(map) => map.get(&async_graphql::Name::new(token))?,
                _ => return None,
            };
        }
        Some(val)
    }

    fn new(value: Self::Output) -> Self {
        value
    }
}

// Highly micro-optimized and benchmarked version of get_path_all
// Any further changes should be verified with benchmarks
pub fn get_path_all<'a>(
    root: &'a serde_json::Value,
    path: &'a [std::string::String],
    mut vector: Vec<(&'a serde_json::Value, &'a serde_json::Value)>,
) -> Vec<(&'a serde_json::Value, &'a serde_json::Value)> {
    match root {
        serde_json::Value::Array(list) => {
            for value in list {
                vector = get_path_all(value, path, vector);
            }
        }
        serde_json::Value::Object(map) => {
            if let Some((key, tail)) = path.split_first() {
                if let Some(value) = map.get(key) {
                    if tail.is_empty() {
                        vector.push((value, root));
                    } else {
                        vector = get_path_all(value, tail, vector);
                    }
                }
            }
        }
        _ => (),
    }

    vector
}

pub fn make_hash_map<'a>(
    src: Vec<(&'a serde_json::Value, &'a serde_json::Value)>,
) -> HashMap<&'a String, Vec<&'a serde_json::Value>> {
    let mut map: HashMap<&'a String, Vec<&'a serde_json::Value>> = HashMap::new();
    for (key, value) in src {
        if let serde_json::Value::String(key) = key {
            if let Some(values) = map.get_mut(key) {
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

    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::json::{json_like::get_path_all, make_hash_map};

    #[test]
    fn test_get_path_all() {
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

        let actual = serde_json::to_value(get_path_all(
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
    fn test_make_hash_map() {
        let arr = vec![
            (json!("1"), json!({"id": "1"})),
            (json!("2"), json!({"id": "2"})),
            (json!("2"), json!({"id": "2"})),
            (json!("3"), json!({"id": "3"})),
        ];
        let input: Vec<(&serde_json::Value, &serde_json::Value)> = arr.iter().map(|(k, v)| (k, v)).collect();

        let actual = serde_json::to_value(make_hash_map(input)).unwrap();

        let expected = json!(
            {
                "1": [{"id": "1"}],
                "2": [{"id": "2"}, {"id": "2"}],
                "3": [{"id": "3"}],
            }
        );

        assert_eq!(actual, expected)
    }
}
