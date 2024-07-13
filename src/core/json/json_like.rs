use std::collections::HashMap;

use async_graphql_value::ConstValue;
use serde_json_borrow::{ObjectAsVec, Value as BorrowedValue};
use simd_json::borrowed::Object as SimdObject;
use simd_json::{BorrowedValue as SimdBorrowedValue, StaticNode};

use crate::core::json::json_object_like::JsonObjectLike;

pub trait JsonLike {
    type Json;
    type JsonObject: JsonObjectLike;

    // Constructors
    fn null() -> Self
    where
        Self: Sized;
    fn new_array(arr: Vec<Self::Json>) -> Self
    where
        Self: Sized;
    fn new(value: &Self::Json) -> &Self
    where
        Self: Sized;

    // Operators
    fn as_array_ok(&self) -> Result<&Vec<Self::Json>, &str>;
    fn as_object_ok(&self) -> Result<&Self::JsonObject, &str>;
    fn as_str_ok(&self) -> Result<&str, &str>;
    fn as_i64_ok(&self) -> Result<i64, &str>;
    fn as_u64_ok(&self) -> Result<u64, &str>;
    fn as_f64_ok(&self) -> Result<f64, &str>;
    fn as_bool_ok(&self) -> Result<bool, &str>;
    fn as_null_ok(&self) -> Result<(), &str>;
    fn as_option_ok(&self) -> Result<Option<&Self::Json>, &str>;
    fn get_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&Self::Json>;
    fn get_key(&self, path: &str) -> Option<&Self::Json>;
    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Json>>;
}

impl JsonLike for serde_json::Value {
    type Json = serde_json::Value;
    type JsonObject = serde_json::Map<String, serde_json::Value>;

    fn as_array_ok(&self) -> Result<&Vec<Self::Json>, &str> {
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

    fn as_option_ok(&self) -> Result<Option<&Self::Json>, &str> {
        match self {
            serde_json::Value::Null => Ok(None),
            _ => Ok(Some(self)),
        }
    }

    fn get_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&Self::Json> {
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

    fn new(value: &Self::Json) -> &Self {
        value
    }

    fn get_key(&self, path: &str) -> Option<&Self::Json> {
        match self {
            serde_json::Value::Object(map) => map.get(path),
            _ => None,
        }
    }

    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Json>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }

    fn null() -> Self {
        Self::Null
    }

    fn new_array(arr: Vec<Self::Json>) -> Self {
        Self::Array(arr)
    }

    fn as_object_ok(&self) -> Result<&Self::JsonObject, &str> {
        match self {
            serde_json::Value::Object(map) => Ok(map),
            _ => Err("expected object"),
        }
    }
}

impl JsonLike for async_graphql::Value {
    type Json = async_graphql::Value;
    type JsonObject = indexmap::IndexMap<async_graphql::Name, async_graphql::Value>;

    fn as_array_ok(&self) -> Result<&Vec<Self::Json>, &str> {
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

    fn as_option_ok(&self) -> Result<Option<&Self::Json>, &str> {
        match self {
            ConstValue::Null => Ok(None),
            _ => Ok(Some(self)),
        }
    }

    fn get_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&Self::Json> {
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

    fn new(value: &Self::Json) -> &Self {
        value
    }

    fn get_key(&self, path: &str) -> Option<&Self::Json> {
        match self {
            ConstValue::Object(map) => map.get(&async_graphql::Name::new(path)),
            _ => None,
        }
    }
    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Json>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }

    fn null() -> Self {
        Default::default()
    }

    fn new_array(arr: Vec<Self::Json>) -> Self {
        ConstValue::List(arr)
    }

    fn as_object_ok(&self) -> Result<&Self::JsonObject, &str> {
        match self {
            ConstValue::Object(map) => Ok(map),
            _ => Err("expected object"),
        }
    }
}

impl<'ctx> JsonLike for BorrowedValue<'ctx> {
    type Json = BorrowedValue<'ctx>;
    type JsonObject = ObjectAsVec<'ctx>;

    fn null() -> Self {
        Self::Null
    }

    fn new_array(arr: Vec<Self::Json>) -> Self {
        Self::Array(arr)
    }

    fn new(value: &Self::Json) -> &Self {
        value
    }

    fn as_array_ok(&self) -> Result<&Vec<Self::Json>, &str> {
        match self {
            BorrowedValue::Array(arr) => Ok(arr),
            _ => Err("expected array"),
        }
    }

    fn as_object_ok(&self) -> Result<&Self::JsonObject, &str> {
        match self {
            BorrowedValue::Object(map) => Ok(map),
            _ => Err("expected object"),
        }
    }

    fn as_str_ok(&self) -> Result<&str, &str> {
        match self {
            BorrowedValue::Str(s) => Ok(s),
            _ => Err("expected string"),
        }
    }

    fn as_i64_ok(&self) -> Result<i64, &str> {
        match self {
            BorrowedValue::Number(n) => n.as_i64().ok_or("expected i64"),
            _ => Err("expected number"),
        }
    }

    fn as_u64_ok(&self) -> Result<u64, &str> {
        match self {
            BorrowedValue::Number(n) => n.as_u64().ok_or("expected u64"),
            _ => Err("expected number"),
        }
    }

    fn as_f64_ok(&self) -> Result<f64, &str> {
        match self {
            BorrowedValue::Number(n) => n.as_f64().ok_or("expected f64"),
            _ => Err("expected number"),
        }
    }

    fn as_bool_ok(&self) -> Result<bool, &str> {
        match self {
            BorrowedValue::Bool(b) => Ok(*b),
            _ => Err("expected bool"),
        }
    }

    fn as_null_ok(&self) -> Result<(), &str> {
        match self {
            BorrowedValue::Null => Ok(()),
            _ => Err("expected null"),
        }
    }

    fn as_option_ok(&self) -> Result<Option<&Self::Json>, &str> {
        match self {
            BorrowedValue::Null => Ok(None),
            _ => Ok(Some(self)),
        }
    }

    fn get_path<T: AsRef<str>>(&self, _path: &[T]) -> Option<&Self::Json> {
        todo!()
    }

    fn get_key(&self, _path: &str) -> Option<&Self::Json> {
        todo!()
    }

    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Json>> {
        let src = gather_path_matches(self, path, vec![]);
        group_by_key(src)
    }
}

impl<'ctx> JsonLike for SimdBorrowedValue<'ctx> {
    type Json = SimdBorrowedValue<'ctx>;
    type JsonObject = SimdObject<'ctx>;

    fn null() -> Self {
        Self::Static(StaticNode::Null)
    }

    fn new_array(arr: Vec<Self::Json>) -> Self {
        Self::Array(arr)
    }

    fn new(value: &Self::Json) -> &Self {
        value
    }

    fn as_array_ok(&self) -> Result<&Vec<Self::Json>, &str> {
        match self {
            SimdBorrowedValue::Array(arr) => Ok(arr),
            _ => Err("expected array"),
        }
    }

    fn as_object_ok(&self) -> Result<&Self::JsonObject, &str> {
        match self {
            SimdBorrowedValue::Object(map) => Ok(map),
            _ => Err("expected object"),
        }
    }

    fn as_str_ok(&self) -> Result<&str, &str> {
        match self {
            SimdBorrowedValue::String(s) => Ok(s),
            _ => Err("expected string"),
        }
    }

    fn as_i64_ok(&self) -> Result<i64, &str> {
        match self {
            SimdBorrowedValue::Static(StaticNode::I64(n)) => Ok(*n),
            _ => Err("expected number"),
        }
    }

    fn as_u64_ok(&self) -> Result<u64, &str> {
        match self {
            SimdBorrowedValue::Static(StaticNode::U64(n)) => Ok(*n),
            _ => Err("expected number"),
        }
    }

    fn as_f64_ok(&self) -> Result<f64, &str> {
        match self {
            SimdBorrowedValue::Static(StaticNode::F64(n)) => Ok(*n),
            _ => Err("expected number"),
        }
    }

    fn as_bool_ok(&self) -> Result<bool, &str> {
        match self {
            SimdBorrowedValue::Static(StaticNode::Bool(b)) => Ok(*b),
            _ => Err("expected bool"),
        }
    }

    fn as_null_ok(&self) -> Result<(), &str> {
        match self {
            SimdBorrowedValue::Static(StaticNode::Null) => Ok(()),
            _ => Err("expected null"),
        }
    }

    fn as_option_ok(&self) -> Result<Option<&Self::Json>, &str> {
        match self {
            SimdBorrowedValue::Static(StaticNode::Null) => Ok(None),
            _ => Ok(Some(self)),
        }
    }

    fn get_path<T: AsRef<str>>(&self, _path: &[T]) -> Option<&Self::Json> {
        todo!()
    }

    fn get_key(&self, _path: &str) -> Option<&Self::Json> {
        todo!()
    }

    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Json>> {
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
            .as_str_ok()
            .map(ToString::to_string)
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
