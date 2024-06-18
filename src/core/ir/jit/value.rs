use serde_json_borrow::Value;

pub trait ValueLike: Clone {
    fn default() -> Self;
    fn path<T: AsRef<str>>(self, path: &[T]) -> Option<Self>;
}

impl<'a> ValueLike for Value<'a> {
    fn default() -> Self {
        Value::Null
    }

    fn path<T: AsRef<str>>(self, tail: &[T]) -> Option<Value<'a>> {
        if tail.is_empty() {
            Some(self)
        } else if let Some((head, tail)) = tail.split_first() {
            match self {
                Value::Null => None,
                Value::Bool(_) => None,
                Value::Number(_) => None,
                Value::Str(_) => None,
                Value::Array(value) => {
                    if let Ok(i) = head.as_ref().parse::<usize>() {
                        value.get(i).and_then(|value| value.clone().path(tail))
                    } else {
                        None
                    }
                }
                Value::Object(obj_vec) => {
                    obj_vec.into_vec().into_iter().find_map(|(key, value)| {
                        if head.as_ref().eq(&key) {
                            value.path(tail)
                        } else {
                            None
                        }
                    })
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;

    fn value() -> Value<'static> {
        let value = serde_json_borrow::OwnedValue::from_str(
            r#"
        {
            "a": {
                "b": {
                    "c": [1, 2, 3]
                }
            }
        }
        "#,
        )
        .unwrap();
        let value = value.deref();
        value.clone()
    }

    #[test]
    fn test_value_like() {
        insta::assert_snapshot!(value().path(&["a"]).unwrap_or(Value::Null).to_string());
    }
    #[test]
    fn test_value_like_nested() {
        insta::assert_snapshot!(value()
            .path(&["a", "b", "c"])
            .unwrap_or(Value::Null)
            .to_string());
    }
    #[test]
    fn test_value_like_list() {
        insta::assert_snapshot!(value()
            .path(&["a", "b", "c", "0"])
            .unwrap_or(Value::Null)
            .to_string());
    }
}
