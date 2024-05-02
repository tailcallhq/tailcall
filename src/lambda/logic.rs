/// Check if a value is truthy
///
/// Special cases:
/// 1. An empty string is considered falsy
/// 2. A collection of bytes is truthy, even if the value in those bytes is 0.
///    An empty collection is falsy.
pub fn is_truthy(value: &async_graphql::Value) -> bool {
    use async_graphql::{Number, Value};
    use hyper::body::Bytes;

    match value {
        &Value::Null => false,
        &Value::Enum(_) => true,
        &Value::List(_) => true,
        &Value::Object(_) => true,
        Value::String(s) => !s.is_empty(),
        &Value::Boolean(b) => b,
        Value::Number(n) => n != &Number::from(0),
        Value::Binary(b) => b != &Bytes::default(),
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::{Name, Number, Value};
    use hyper::body::Bytes;
    use indexmap::IndexMap;

    use crate::lambda::is_truthy;

    #[test]
    fn test_is_truthy() {
        assert!(is_truthy(&Value::Enum(Name::new("EXAMPLE"))));
        assert!(is_truthy(&Value::List(vec![])));
        assert!(is_truthy(&Value::Object(IndexMap::default())));
        assert!(is_truthy(&Value::String("Hello".to_string())));
        assert!(is_truthy(&Value::Boolean(true)));
        assert!(is_truthy(&Value::Number(Number::from(1))));
        assert!(is_truthy(&Value::Binary(Bytes::from_static(&[0, 1, 2]))));

        assert!(!is_truthy(&Value::Null));
        assert!(!is_truthy(&Value::String("".to_string())));
        assert!(!is_truthy(&Value::Boolean(false)));
        assert!(!is_truthy(&Value::Number(Number::from(0))));
        assert!(!is_truthy(&Value::Binary(Bytes::default())));
    }
}
