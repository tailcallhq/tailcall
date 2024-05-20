use fxhash::FxHashMap;
use serde::de::{self};

use crate::schema::{self, Schema};
use crate::value;

type Output = crate::Value;

struct FieldSchema<'de> {
    name: &'de str,
    schema: &'de Schema,
}

struct Row(Vec<Output>);

type ObjectMap = FxHashMap<String, Schema>;

struct Object<'de>(&'de ObjectMap);

impl Object<'_> {
    pub fn new<'de>(fields: &'de ObjectMap) -> Object<'de> {
        Object(&fields)
    }
}

impl<'de> de::Visitor<'de> for Object<'de> {
    type Value = Option<FieldSchema<'de>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a field name")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match self.0.get_key_value(v) {
            Some((name, schema)) => Ok(Some(FieldSchema { name, schema })),
            None => Ok(None),
        }
    }
}

impl<'de> de::DeserializeSeed<'de> for Object<'de> {
    type Value = Option<FieldSchema<'de>>;
    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let visitor = Object::new(self.0);
        deserializer.deserialize_identifier(visitor)
    }
}

pub struct Value<'de> {
    schema: &'de Schema,
}

impl<'de> Value<'de> {
    pub fn new(schema: &'de Schema) -> Self {
        Self { schema }
    }
}

impl<'de> de::Visitor<'de> for Value<'de> {
    type Value = Output;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.schema {
            Schema::Primitive(schema) => match schema {
                schema::Primitive::String => formatter.write_str("a string"),
                schema::Primitive::Boolean => formatter.write_str("a boolean"),
                schema::Primitive::Number(n) => match n {
                    schema::N::I64 => formatter.write_str("a i64"),
                    schema::N::U64 => formatter.write_str("a u64"),
                    schema::N::F64 => formatter.write_str("a f64"),
                },
            },
            Schema::Object(_) => formatter.write_str("an object"),
            Schema::Table { map: _, head: _, rows: _ } => formatter.write_str("a table"),
            Schema::Array(_) => formatter.write_str("an array"),
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Output::from_string(value.to_owned()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Output::from_string(v))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Output::from_bool(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Output::from_f64(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Output::from_u64(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Output::from_i64(v))
    }

    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        if let Schema::Object(fields) = self.schema {
            let mut rows = Vec::with_capacity(fields.len());
            while let Some(field) = map.next_key_seed(Object::new(fields))? {
                match field {
                    Some(field) => {
                        let value_schema = field.schema;
                        match map.next_value_seed(Value::new(&value_schema)) {
                            Ok(value) => rows.push((field.name.to_owned(), value)),
                            Err(err) => return Err(err),
                        };
                    }
                    None => {
                        let _: de::IgnoredAny = map.next_value()?;
                    }
                }
            }

            Ok(Output::Object(rows))
        } else {
            Err(de::Error::custom("expected object"))
        }
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        match self.schema {
            Schema::Table { rows: _, head, map } => {
                let mut rows = Vec::with_capacity(seq.size_hint().unwrap_or(100));

                while let Ok(Some(row)) = seq.next_element_seed(Table::new(map)) {
                    rows.push(row.0);
                }

                Ok(Output::Table { head: head.to_owned(), rows })
            }
            Schema::Array(primitive) => {
                let mut rows = Vec::with_capacity(seq.size_hint().unwrap_or(100));
                while let Ok(Some(row)) = seq.next_element_seed(Primitive::new(primitive)) {
                    rows.push(row);
                }

                Ok(Output::Array(rows))
            }
            _ => Err(de::Error::custom("expected a table or an array")),
        }
    }
}

impl<'de> de::DeserializeSeed<'de> for Value<'de> {
    type Value = Output;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let visitor = Value::new(self.schema);
        match &self.schema {
            Schema::Primitive(schema) => match schema {
                schema::Primitive::Boolean => deserializer.deserialize_bool(visitor),
                schema::Primitive::Number(n) => match n {
                    schema::N::I64 => deserializer.deserialize_i64(visitor),
                    schema::N::U64 => deserializer.deserialize_u64(visitor),
                    schema::N::F64 => deserializer.deserialize_f64(visitor),
                },
                schema::Primitive::String => deserializer.deserialize_str(visitor),
            },
            Schema::Object(_) => deserializer.deserialize_map(visitor),
            Schema::Table { map: _, head: _, rows: _ } => deserializer.deserialize_seq(visitor),
            Schema::Array(_) => deserializer.deserialize_seq(visitor),
        }
    }
}

struct Table<'de>(&'de ObjectMap);

impl<'de> Table<'de> {
    pub fn new(fields: &'de ObjectMap) -> Self {
        Self(fields)
    }
}

impl<'de> de::DeserializeSeed<'de> for Table<'de> {
    type Value = Row;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de> de::Visitor<'de> for Table<'de> {
    type Value = Row;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a row")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut cols = Vec::with_capacity(self.0.len());
        while let Some(field) = map.next_key_seed(Object::new(self.0))? {
            match field {
                Some(field) => {
                    let schema = field.schema;
                    match map.next_value_seed(Value::new(&schema)) {
                        Ok(value) => cols.push(value),
                        Err(err) => return Err(err),
                    }
                }

                None => {
                    let _: de::IgnoredAny = map.next_value()?;
                }
            }
        }

        Ok(Row(cols))
    }
}

struct Primitive<'de>(&'de schema::Primitive);

impl<'de> Primitive<'de> {
    fn new(schema: &'de schema::Primitive) -> Self {
        Self(schema)
    }
}

impl<'de> de::Visitor<'de> for Primitive<'de> {
    type Value = value::Primitive;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.0 {
            schema::Primitive::String => formatter.write_str("a string"),
            schema::Primitive::Boolean => formatter.write_str("a boolean"),
            schema::Primitive::Number(n) => match n {
                schema::N::I64 => formatter.write_str("a i64"),
                schema::N::U64 => formatter.write_str("a u64"),
                schema::N::F64 => formatter.write_str("a f64"),
            },
        }
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(value::Primitive::from_string(value.to_owned()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value::Primitive::from_string(v))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value::Primitive::from_bool(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value::Primitive::from_f64(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value::Primitive::from_u64(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value::Primitive::from_i64(v))
    }
}

impl<'de> de::DeserializeSeed<'de> for Primitive<'de> {
    type Value = value::Primitive;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        match self.0 {
            schema::Primitive::String => deserializer.deserialize_str(self),
            schema::Primitive::Boolean => deserializer.deserialize_bool(self),
            schema::Primitive::Number(n) => match n {
                schema::N::I64 => deserializer.deserialize_i64(self),
                schema::N::U64 => deserializer.deserialize_u64(self),
                schema::N::F64 => deserializer.deserialize_f64(self),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;

    use super::*;
    use crate::schema::Schema;

    #[test]
    fn test_string() {
        let schema = Schema::string();
        let input = r#""Hello World!""#;
        let actual = schema.from_str(input).unwrap();
        assert_snapshot!(actual);
    }

    #[test]
    fn test_bool() {
        let schema = Schema::boolean();
        let input = r#"true"#;
        let actual = schema.from_str(input).unwrap();
        assert_snapshot!(actual);
    }

    #[test]
    fn test_i32() {
        let schema = Schema::u64();
        let actual = schema.from_str(r#"42"#).unwrap();
        assert_snapshot!(actual);

        let actual = schema.from_str(r#"-42"#).unwrap();
        assert_snapshot!(actual);
    }

    #[test]
    fn test_u64() {
        let schema = Schema::u64();
        let input = r#"42"#;
        let actual = schema.from_str(input).unwrap();
        assert_snapshot!(actual);
    }

    #[test]
    fn test_f64() {
        let schema = Schema::f64();
        let input = r#"42.0"#;
        let actual = schema.from_str(input).unwrap();
        assert_snapshot!(actual);
    }

    #[test]
    fn test_object() {
        let schema = Schema::object(&[(("foo", Schema::u64())), (("bar", Schema::boolean()))]);
        let input = r#"{"foo": 42, "bar": true}"#;
        let actual = schema.from_str(input).unwrap();
        assert_snapshot!(actual);
    }

    #[test]
    fn test_object_partial() {
        let schema = Schema::object(&[(("foo", Schema::u64()))]);
        let input = r#"{"foo": 42, "bar": true}"#;
        let actual = schema.from_str(input).unwrap();
        assert_snapshot!(actual);
    }

    #[test]
    #[ignore]
    fn test_object_missing() {
        let schema = Schema::object(&[(("foo", Schema::u64()))]);
        let input = r#"{"bar": true}"#;
        let actual = schema.from_str(input).err().unwrap();
        assert_snapshot!(actual);
    }

    #[test]
    #[ignore]
    fn test_object_order() {
        let schema = Schema::object(&[(("bar", Schema::boolean())), (("foo", Schema::u64()))]);
        let input = r#"{"foo": 42, "bar": true}"#;
        let actual = schema.from_str(input).unwrap();
        assert_snapshot!(actual);
    }

    #[test]
    fn test_array() {
        let schema = Schema::array(schema::Primitive::u64());
        let input = r#"[1, 2, 3]"#;
        let actual = schema.from_str(input).unwrap();
        assert_snapshot!(actual);
    }

    #[test]
    fn test_table() {
        let schema = Schema::table(&[("foo", Schema::u64()), ("bar", Schema::string())]);
        let input = r#"[{"foo":1,"bar":"Hello"},{"foo":2,"bar":"Bye"}]"#;
        let actual = schema.from_str(input).unwrap();
        assert_snapshot!(actual);
    }

    #[test]
    fn test_table_partial() {
        let schema = Schema::table(&[("foo", Schema::u64())]);
        let input = r#"[{"foo":1,"bar":"Hello"},{"foo":2,"bar":"Bye"}]"#;
        let actual = schema.from_str(input).unwrap();
        assert_snapshot!(actual);
    }

    #[test]
    #[ignore]
    fn test_table_missing() {
        let schema = Schema::table(&[("foo", Schema::u64())]);
        let input = r#"[{"bar":"Hello"},{"bar":"Bye"}]"#;
        let actual = schema.from_str(input).err().unwrap();
        assert_snapshot!(actual);
    }

    #[test]
    #[ignore]
    fn test_table_order() {
        let schema = Schema::table(&[("bar", Schema::string()), ("foo", Schema::u64())]);
        let input = r#"[{"foo":1,"bar":"Hello"},{"foo":2,"bar":"Bye"}]"#;
        let actual = schema.from_str(input).unwrap();
        assert_snapshot!(actual);
    }

    #[test]
    fn test_posts() {
        const JSON: &str = include_str!("../data/posts.json");
        let schema = Schema::table(&[
            // ("user_id", Schema::u64()),
            ("id", Schema::u64()),
            // ("title", Schema::string()),
            // ("body", Schema::string()),
        ]);
        let actual = schema.from_str(JSON).unwrap();
        assert_snapshot!(actual);
    }
}
