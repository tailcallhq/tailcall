use serde_json_borrow::Value;

pub trait ValueLike: Clone {
    fn default() -> Self;
    fn path(self, path: &[String]) -> Option<Self>;
}

impl<'a> ValueLike for Value<'a> {
    fn path(self, tail: &[String]) -> Option<Value<'a>> {
        if tail.is_empty() {
            Some(self)
        } else if let Some((head, tail)) = tail.split_first() {
            match self {
                Value::Null => None,
                Value::Bool(_) => None,
                Value::Number(_) => None,
                Value::Str(_) => None,
                Value::Array(value) => {
                    if let Ok(i) = head.parse::<usize>() {
                        value.get(i).and_then(|value| value.clone().path(tail))
                    } else {
                        None
                    }
                }
                Value::Object(obj_vec) => obj_vec.iter().find_map(|(key, value)| {
                    if key == head {
                        value.clone().path(tail)
                    } else {
                        None
                    }
                }),
            }
        } else {
            None
        }
    }

    fn default() -> Self {
        Value::Null
    }
}
