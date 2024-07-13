use std::collections::HashMap;

/// A trait for objects that can be used as JSON values
pub trait JsonLike<'a>: Sized {
    type JsonObject: JsonObjectLike<'a, Value = Self>;

    // Constructors
    fn null() -> Self;

    // Operators
    fn as_array_ok(&'a self) -> Result<&'a Vec<Self>, &str>;
    fn as_object_ok(&'a self) -> Result<&Self::JsonObject, &str>;
    fn as_str_ok(&'a self) -> Result<&str, &str>;
    fn as_i64_ok(&'a self) -> Result<i64, &str>;
    fn as_u64_ok(&'a self) -> Result<u64, &str>;
    fn as_f64_ok(&'a self) -> Result<f64, &str>;
    fn as_bool_ok(&'a self) -> Result<bool, &str>;
    fn as_null_ok(&'a self) -> Result<(), &str>;
    fn as_option_ok(&'a self) -> Result<Option<&Self>, &str>;
    fn get_path<T: AsRef<str>>(&'a self, path: &'a [T]) -> Option<&Self>;
    fn get_key(&'a self, path: &'a str) -> Option<&Self>;
    fn group_by(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self>>;
}

/// A trait for objects that can be used as JSON objects
pub trait JsonObjectLike<'a> {
    type Value;
    fn get_key(&'a self, key: &str) -> Option<&Self::Value>;
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
