use std::collections::HashMap;

use async_graphql_value::ConstValue;

pub trait JsonLike {
    type Output;
    fn as_array_ok(&self) -> Result<&Vec<Self::Output>, &str>;
    fn as_string_ok(&self) -> Result<&String, &str>;
    fn as_i64_ok(&self) -> Result<i64, &str>;
    fn as_u64_ok(&self) -> Result<u64, &str>;
    fn as_f64_ok(&self) -> Result<f64, &str>;
    fn get_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&Self::Output>;
    fn get_key(&self, path: &str) -> Option<&Self::Output>;
    fn new(value: &Self::Output) -> &Self;
    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Output>>;
}

impl JsonLike for serde_json::Value {
    type Output = serde_json::Value;
    fn as_array_ok(&self) -> Result<&Vec<Self::Output>, &str> {
        self.as_array().ok_or("expected array")
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

    fn get_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&Self::Output> {
        let mut val = self;
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

    fn new(value: &Self::Output) -> &Self {
        value
    }

    fn get_key(&self, path: &str) -> Option<&Self::Output> {
        match self {
            serde_json::Value::Object(map) => map.get(path),
            _ => None,
        }
    }

    fn as_string_ok(&self) -> Result<&String, &str> {
        match self {
            serde_json::Value::String(s) => Ok(s),
            _ => Err("expected string"),
        }
    }

    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Output>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
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

    fn get_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&Self::Output> {
        let mut val = self;
        for token in path {
            val = match val {
                ConstValue::List(seq) => {
                    let index = token.as_ref().parse::<usize>().ok()?;
                    seq.get(index)?
                }
                ConstValue::Object(map) => map.get(token.as_ref())?,
                _ => return None,
            };
        }
        Some(val)
    }

    fn new(value: &Self::Output) -> &Self {
        value
    }

    fn get_key(&self, path: &str) -> Option<&Self::Output> {
        match self {
            ConstValue::Object(map) => map.get(&async_graphql::Name::new(path)),
            _ => None,
        }
    }
    fn as_string_ok(&self) -> Result<&String, &str> {
        match self {
            ConstValue::String(s) => Ok(s),
            _ => Err("expected string"),
        }
    }

    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Output>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }
}

// Highly micro-optimized and benchmarked version of get_path_all
// Any further changes should be verified with benchmarks
pub fn gather_path_matches<'a, J: JsonLike>(
    root: &'a J,
    path: &'a [String],
    mut vector: Vec<(&'a J, &'a J)>,
) -> Vec<(&'a J, &'a J)> {
    if let Ok(root) = root.as_array_ok() {
        for value in root {
            vector = gather_path_matches(J::new(value), path, vector);
        }
    } else if let Some((key, tail)) = path.split_first() {
        if let Some(value) = root.get_key(key) {
            if tail.is_empty() {
                vector.push((J::new(value), root));
            } else {
                vector = gather_path_matches(J::new(value), tail, vector);
            }
        }
    }

    vector
}

pub fn group_by_key<'a, J: JsonLike>(src: Vec<(&'a J, &'a J)>) -> HashMap<String, Vec<&'a J>> {
    let mut map: HashMap<String, Vec<&'a J>> = HashMap::new();
    for (key, value) in src {
        // Need to handle number and string keys
        let key_str = key
            .as_string_ok()
            .cloned()
            .or_else(|_| key.as_f64_ok().map(|a| a.to_string()));

        if let Ok(key) = key_str {
            if let Some(values) = map.get_mut(&key) {
                values.push(value);
            } else {
                map.insert(key, vec![value]);
            }
        }
    }
    map
}
