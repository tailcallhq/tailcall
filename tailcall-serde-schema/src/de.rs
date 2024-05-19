use serde::de::{self};
use serde_json::{self};

use crate::ignore::Ignore;
use crate::schema::Schema;

pub struct Deserialize<'de> {
    schema: &'de Schema,
}

impl<'de> Deserialize<'de> {
    pub fn new(schema: &'de Schema) -> Self {
        Self { schema }
    }
}

impl<'de> de::DeserializeSeed<'de> for Deserialize<'de> {
    type Value = serde_json::Value;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let visitor = Visitor::new(self.schema);
        match &self.schema {
            Schema::Boolean => deserializer.deserialize_bool(visitor),
            Schema::Number(n) => match n {
                crate::schema::N::I64 => deserializer.deserialize_i64(visitor),
                crate::schema::N::U64 => deserializer.deserialize_u64(visitor),
                crate::schema::N::F64 => deserializer.deserialize_f64(visitor),
            },
            Schema::String => deserializer.deserialize_str(visitor),
            Schema::Object(_) => deserializer.deserialize_map(visitor),
            Schema::Array(_) => deserializer.deserialize_seq(visitor),
        }
    }
}

struct Visitor<'de> {
    schema: &'de Schema,
}

impl<'de> Visitor<'de> {
    pub fn new(schema: &'de Schema) -> Self {
        Self { schema }
    }
}

impl<'de> serde::de::Visitor<'de> for Visitor<'de> {
    type Value = serde_json::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.schema {
            Schema::String => formatter.write_str("a string"),
            Schema::Boolean => formatter.write_str("a boolean"),
            Schema::Number(n) => match n {
                crate::schema::N::I64 => formatter.write_str("a i64"),
                crate::schema::N::U64 => formatter.write_str("a u64"),
                crate::schema::N::F64 => formatter.write_str("a f64"),
            },
            Schema::Object(_) => formatter.write_str("an object"),
            Schema::Array(_) => formatter.write_str("an array"),
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(serde_json::Value::String(value.to_string()))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(serde_json::Value::Bool(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(serde_json::Value::Number(
            serde_json::Number::from_f64(v).unwrap(),
        ))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(serde_json::Value::Number(serde_json::Number::from(v)))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(serde_json::Value::Number(serde_json::Number::from(v)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        if let Schema::Object(fields) = self.schema {
            let mut object = serde_json::Map::new();
            while let Some(key) = map.next_key::<&str>()? {
                if let Some(value_schema) = fields.get(key) {
                    let value = map.next_value_seed(Deserialize::new(value_schema))?;
                    object.insert(key.to_string(), value);
                } else {
                    map.next_value_seed(Ignore)?;
                }
            }

            Ok(serde_json::Value::Object(object))
        } else {
            Err(de::Error::custom("expected object"))
        }
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        if let Schema::Array(item) = self.schema {
            let mut array = Vec::new();
            while let Some(value) = seq.next_element_seed(Deserialize::new(item))? {
                array.push(value);
            }

            Ok(serde_json::Value::Array(array))
        } else {
            Err(de::Error::custom("expected array"))
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::schema::{Schema, N};

    fn check_schema(schema: &Schema, input: &str) {
        let actual = schema.deserialize(input).unwrap();
        let expected = serde_json::from_str::<serde_json::Value>(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_string() {
        let schema = Schema::String;
        let input = r#""Hello World!""#;
        check_schema(&schema, input);
    }

    #[test]
    fn test_bool() {
        let schema = Schema::Boolean;
        let input = r#"true"#;
        check_schema(&schema, input);
    }

    #[test]
    fn test_i32() {
        let schema = Schema::Number(N::U64);
        check_schema(&schema, r#"42"#);
        check_schema(&schema, r#"-42"#);
    }

    #[test]
    fn test_u64() {
        let schema = Schema::Number(N::U64);
        let input = r#"42"#;
        check_schema(&schema, input);
    }

    #[test]
    fn test_f64() {
        let schema = Schema::Number(N::F64);
        let input = r#"42.0"#;
        check_schema(&schema, input);
    }

    #[test]
    fn test_object() {
        let schema = Schema::Object({
            let mut fields = std::collections::HashMap::new();
            fields.insert("foo".to_string(), Box::new(Schema::Number(N::U64)));
            fields.insert("bar".to_string(), Box::new(Schema::Boolean));
            fields
        });
        let input = r#"{"foo": 42, "bar": true}"#;
        check_schema(&schema, input);
    }

    #[test]
    fn test_array() {
        let schema = Schema::array(Schema::Number(N::U64));
        let input = r#"[1, 2, 3]"#;
        check_schema(&schema, input);
    }
}
