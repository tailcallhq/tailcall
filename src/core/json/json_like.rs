use std::borrow::Cow;
use std::collections::HashMap;

use crate::core::ir::model::{InputTransforms, TransformKey};

pub trait JsonLikeOwned: for<'json> JsonLike<'json> {}
impl<T> JsonLikeOwned for T where T: for<'json> JsonLike<'json> {}

/// A trait for objects that can be used as JSON values
pub trait JsonLike<'json>: Sized + Clone {
    type JsonObject<'obj>: JsonObjectLike<
        'obj,
        // generally we want to specify `Self` instead of generic here
        // and `Self` is used anyway through JsonObjectLike for
        // current implementations.
        // But `Self` means the very specific type with some specific lifetime
        // which doesn't work in case we want to return self type but with different
        // lifetime. Currently, it affects only `as_object` fn because `serde_json_borrow`
        // returns smaller lifetime for Value in its `as_object` fn that either forces to
        // use `&'json self` in the fn (that leads to error "variable does not live long enough")
        // or generic like this.
        // TODO: perhaps it could be fixed on `serde_json_borrow` side if we return `Value<'ctx>`
        // instead of `Value<'_>` in its functions like `as_object`. In that case we can specify
        // `Self` here and simplify usages of this trait
        Value: JsonLike<'obj>,
    >;

    // Constructors
    fn null() -> Self;
    fn object(obj: Self::JsonObject<'json>) -> Self;
    fn array(arr: Vec<Self>) -> Self;
    fn string(s: Cow<'json, str>) -> Self;

    // Operators
    fn as_array(&self) -> Option<&Vec<Self>>;
    fn as_object(&self) -> Option<&Self::JsonObject<'_>>;
    fn as_str(&self) -> Option<&str>;
    fn as_i64(&self) -> Option<i64>;
    fn as_u64(&self) -> Option<u64>;
    fn as_f64(&self) -> Option<f64>;
    fn as_bool(&self) -> Option<bool>;
    fn is_null(&self) -> bool;
    fn get_path<T: AsRef<str>>(&'json self, path: &[T]) -> Option<&Self>;
    fn get_key(&'json self, path: &str) -> Option<&Self>;
    fn group_by(&'json self, path: &[String]) -> HashMap<String, Vec<&Self>>;

    ///
    /// Used to apply `input_transforms` operations on the given `json_like`
    /// struct. Current supported operations are `modify` - rename.
    fn handle_input_transforms(
        &'json self,
        input_transforms: &'json InputTransforms,
        type_of: &'json str,
    ) -> Self {
        if let Some(items) = self.as_array() {
            // if: it is an array, we iterate each item and we call recursively the
            // `handle_input_transforms` to apply the transform for each item.
            let arr = items
                .iter()
                .clone()
                .map(|item| item.handle_input_transforms(input_transforms, type_of))
                .collect::<Vec<_>>();
            Self::array(arr)
        } else if let Some(obj) = self.as_object() {
            // if: it is an object, we iterate the fields of the object and we apply the
            // various transformations on the value
            let mut new_map = Self::JsonObject::new();

            // iter: every object field and apply transformations or go deeper to the next
            // object type recursively
            for (name, item) in obj.get_iterator() {
                let key = TransformKey::from_str(type_of.to_string(), name.to_string());
                let type_new = input_transforms.subfield_types.get(&key);
                let name_new = input_transforms.subfield_renames.get(&key);

                match (type_new, name_new) {
                    // if: basic type, no rename
                    (None, None) => new_map.insert_key(
                        name,
                        item.handle_input_transforms(input_transforms, type_of),
                    ),
                    // if: basic type, rename
                    (None, Some(name_new)) => new_map.insert_key(
                        name_new,
                        item.handle_input_transforms(input_transforms, type_of),
                    ),
                    // if: complex type type, no rename
                    (Some(type_new), None) => new_map.insert_key(
                        name,
                        item.handle_input_transforms(input_transforms, type_new),
                    ),
                    // if: complex type type, rename
                    (Some(type_new), Some(name_new)) => new_map.insert_key(
                        name_new,
                        item.handle_input_transforms(input_transforms, type_new),
                    ),
                };
            }

            Self::object(new_map)
        } else {
            // if: anything else we just return it
            self.clone()
        }
    }
}

/// A trait for objects that can be used as JSON objects
pub trait JsonObjectLike<'obj>: Sized {
    type Value;
    fn new() -> Self;
    fn get_key(&'obj self, key: &str) -> Option<&Self::Value>;
    fn insert_key(&mut self, key: &'obj str, value: Self::Value);
    fn get_iterator(&'obj self) -> impl Iterator<Item = (&'obj str, &'obj Self::Value)>;
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
