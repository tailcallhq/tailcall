use std::collections::HashMap;

/*pub trait JsonLikeOwned: for<'json> JsonLike<'json> {}
impl<T> JsonLikeOwned for T where T: for<'json> JsonLike<'json> {}*/

/// A trait for objects that can be used as JSON values
pub trait JsonLike: Sized {
    type JsonObject<'a>: JsonObjectLike<'a>
    where
        Self: 'a;
    type Output<'a>: JsonLike
    where
        Self: 'a;

    // Constructors
    fn null() -> Self;

    // Operators
    fn as_array(&self) -> Option<&Vec<Self>>;
    fn as_object(&self) -> Option<&Self::JsonObject<'_>>;
    fn as_str(&self) -> Option<&str>;
    fn as_i64(&self) -> Option<i64>;
    fn as_u64(&self) -> Option<u64>;
    fn as_f64(&self) -> Option<f64>;
    fn as_bool(&self) -> Option<bool>;
    fn is_null(&self) -> bool;
    fn get_path<'a, T: AsRef<str>>(&'a self, path: &'a [T]) -> Option<&Self::Output<'a>>;
    fn get_key<'a>(&'a self, path: &'a str) -> Option<&Self::Output<'a>>;
    fn group_by<'a>(&'a self, path: &'a [String]) -> HashMap<String, Vec<&'a Self::Output<'a>>>;
}

/// A trait for objects that can be used as JSON objects
pub trait JsonObjectLike<'ctx>: Sized {
    type Value<'json>
    where
        Self: 'json,
        'json: 'ctx;

    fn new() -> Self;
    fn get_key<'a: 'ctx>(&'a self, key: &str) -> Option<&Self::Value<'a>>;
    fn insert_key<'a: 'ctx>(&mut self, key: &'a str, value: Self::Value<'a>)
    where
        Self: 'a;
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
