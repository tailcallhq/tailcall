use std::borrow::Cow;
use std::collections::HashMap;

pub enum JsonPrimitive<'a> {
    Null,
    Bool(bool),
    Str(&'a str),
    Number(serde_json::Number),
}

pub trait JsonLikeOwned: for<'json> JsonLike<'json> {}
impl<T> JsonLikeOwned for T where T: for<'json> JsonLike<'json> {}

/// A trait for objects that can be used as JSON values
pub trait JsonLike<'json>: Sized {
    type JsonObject: JsonObjectLike<'json, Value = Self>;

    // Constructors
    fn null() -> Self;
    fn object(obj: Self::JsonObject) -> Self;
    fn array(arr: Vec<Self>) -> Self;
    fn string(s: Cow<'json, str>) -> Self;
    fn from_primitive(x: JsonPrimitive<'json>) -> Self;
    fn clone_from<T>(other: &'json T) -> Self
    where
        T: JsonLike<'json>,
        T::JsonObject: JsonObjectLike<'json, Value = T>,
    {
        if let Some(obj) = other.as_object() {
            let mut fields = Vec::with_capacity(obj.len());
            for (k, v) in obj.iter() {
                fields.push((k, Self::clone_from(v)));
            }

            Self::object(Self::JsonObject::from_vec(fields))
        } else if let Some(arr) = other.as_array() {
            let v = arr.iter().map(Self::clone_from).collect();

            Self::array(v)
        } else if let Some(primitive) = other.as_primitive() {
            Self::from_primitive(primitive)
        } else {
            unreachable!()
        }
    }

    // Operators
    fn as_primitive(&self) -> Option<JsonPrimitive>;
    fn as_array(&self) -> Option<&Vec<Self>>;
    fn as_array_mut(&mut self) -> Option<&mut Vec<Self>>;
    fn into_array(self) -> Option<Vec<Self>>;
    fn as_object(&self) -> Option<&Self::JsonObject>;
    fn as_object_mut(&mut self) -> Option<&mut Self::JsonObject>;
    fn into_object(self) -> Option<Self::JsonObject>;
    fn as_str(&self) -> Option<&str>;
    fn as_i64(&self) -> Option<i64>;
    fn as_u64(&self) -> Option<u64>;
    fn as_f64(&self) -> Option<f64>;
    fn as_bool(&self) -> Option<bool>;
    fn is_null(&self) -> bool;
    fn get_path<T: AsRef<str>>(&'json self, path: &[T]) -> Option<&'json Self>;
    fn get_key(&'json self, path: &str) -> Option<&'json Self>;
    fn group_by(&'json self, path: &[String]) -> HashMap<String, Vec<&'json Self>>;
}

/// A trait for objects that can be used as JSON objects
pub trait JsonObjectLike<'obj>: Sized {
    type Value;
    fn new() -> Self;
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn with_capacity(n: usize) -> Self;
    fn from_vec(v: Vec<(&'obj str, Self::Value)>) -> Self;
    fn get_key(&self, key: &str) -> Option<&Self::Value>;
    fn insert_key(&mut self, key: &'obj str, value: Self::Value);
    fn iter<'slf>(&'slf self) -> impl Iterator<Item = (&'slf str, &'slf Self::Value)>
    where
        Self::Value: 'slf,
        'obj: 'slf,
    {
        std::iter::empty()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::super::gather_path_matches;
    use super::{JsonLike, JsonObjectLike};
    use crate::core::json::group_by_key;

    // for lifetime testing purposes
    #[allow(dead_code)]
    fn create_json_like<'a, Value: JsonLike<'a>>() -> Value {
        unimplemented!("fake test fn")
    }

    // for lifetime testing purposes
    #[allow(dead_code)]
    fn test_json_like_lifetime<'a, Value: JsonLike<'a> + Clone>() -> Value {
        let value: Value = create_json_like();

        if value.is_null() {
            return Value::null();
        }

        if value.as_bool().is_some() {
            println!("bool");
        }

        if value.as_f64().is_some() {
            println!("f64");
        }

        if let Some(s) = value.as_str() {
            return Value::string(s.to_string().into());
        }

        if let Some(arr) = value.as_array() {
            return Value::array(arr.clone());
        }

        if value.as_object().is_some() {
            return Value::object(Value::JsonObject::new());
        }

        value
    }

    #[allow(dead_code)]
    fn test_json_object_like_lifetime<'a, Value: JsonObjectLike<'a, Value = bool> + Clone + 'a>(
    ) -> Value {
        let mut value = Value::new();

        value.insert_key("test_key", true);
        let _key = value.get_key("test_key");

        let it1 = value.iter();
        let it2 = value.iter();

        for _val in it1 {}

        for _val in it2 {}

        value
    }

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
