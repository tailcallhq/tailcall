use std::collections::HashMap;

pub trait JsonLikeOwned: for<'json> JsonLike<'json> {}
impl<T> JsonLikeOwned for T where T: for<'json> JsonLike<'json> {}

/// A trait for objects that can be used as JSON values
pub trait JsonLike<'a>: Sized {
    type JsonObject: JsonObjectLike<'a, Value = Self>;

    // Constructors
    fn null() -> Self;
    fn object(obj: Self::JsonObject) -> Self;
    fn array(arr: Vec<Self>) -> Self;
    fn string(s: &'a str) -> Self;

    // Operators
    fn as_array(&'a self) -> Option<&'a Vec<Self>>;
    fn as_object(&'a self) -> Option<&Self::JsonObject>;
    fn as_str(&'a self) -> Option<&str>;
    fn as_i64(&'a self) -> Option<i64>;
    fn as_u64(&'a self) -> Option<u64>;
    fn as_f64(&'a self) -> Option<f64>;
    fn as_bool(&'a self) -> Option<bool>;
    fn is_null(&'a self) -> bool;
    fn get_path<T: AsRef<str>>(&'a self, path: &'a [T]) -> Option<&Self>;
    fn get_key(&'a self, path: &'a str) -> Option<&Self>;
    fn group_by(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self>>;
}

/// A trait for objects that can be used as JSON objects
pub trait JsonObjectLike<'a>: Sized {
    type Value;
    fn new() -> Self;
    fn get_key(&'a self, key: &str) -> Option<&Self::Value>;
    fn insert_key(self, key: &'a str, value: Self::Value) -> Self;
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::super::gather_path_matches;
    use crate::core::json::group_by_key;

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
