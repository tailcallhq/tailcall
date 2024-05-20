use serde::de::{self, IgnoredAny};

use crate::schema::{self, Schema};
use crate::value;

type Value = crate::Value;

pub struct Deserialize<'de> {
    schema: &'de Schema,
}

impl<'de> Deserialize<'de> {
    pub fn new(schema: &'de Schema) -> Self {
        Self { schema }
    }
}

struct Field<'de> {
    name: &'de str,
    schema: &'de Schema,
}

struct FieldSelection<'de> {
    fields: &'de [(String, Schema)],
}

struct FieldVisitor<'de> {
    fields: &'de [(String, Schema)],
}

impl FieldVisitor<'_> {
    pub fn new<'de>(fields: &'de [(String, Schema)]) -> FieldVisitor<'de> {
        FieldVisitor { fields }
    }
}

impl<'de> de::Visitor<'de> for FieldVisitor<'de> {
    type Value = Option<Field<'de>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a field name")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match self.fields.iter().find(|(u, _)| u == v) {
            Some((name, schema)) => Ok(Some(Field { name, schema })),
            None => Ok(None),
        }
    }
}

impl FieldSelection<'_> {
    pub fn new<'de>(fields: &'de [(String, Schema)]) -> FieldSelection<'de> {
        FieldSelection { fields }
    }
}

impl<'de> de::DeserializeSeed<'de> for FieldSelection<'de> {
    type Value = Option<Field<'de>>;
    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let visitor = FieldVisitor::new(self.fields);
        deserializer.deserialize_identifier(visitor)
    }
}

impl<'de> de::DeserializeSeed<'de> for Deserialize<'de> {
    type Value = Value;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let visitor = ValueVisitor::new(self.schema);
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

struct ValueVisitor<'de> {
    schema: &'de Schema,
}

impl<'de> ValueVisitor<'de> {
    pub fn new(schema: &'de Schema) -> Self {
        Self { schema }
    }
}

struct RowVisitor<'de> {
    fields: &'de [(String, Schema)],
}

struct Row {
    cols: Vec<Value>,
}

impl<'de> RowVisitor<'de> {
    pub fn new(fields: &'de [(String, Schema)]) -> Self {
        Self { fields }
    }
}

impl<'de> de::DeserializeSeed<'de> for RowVisitor<'de> {
    type Value = Row;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }
}

impl<'de> de::Visitor<'de> for RowVisitor<'de> {
    type Value = Row;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a row")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut cols = Vec::new();
        let fields = self.fields;
        while let Some(field) = map.next_key_seed(FieldSelection::new(fields))? {
            match field {
                Some(field) => {
                    let schema = field.schema;
                    match map.next_value_seed(Deserialize::new(&schema)) {
                        Ok(value) => cols.push(value),
                        Err(err) => return Err(err),
                    }
                }

                None => {
                    let _: IgnoredAny = map.next_value()?;
                }
            }
        }

        Ok(Row { cols })
    }
}

struct PrimitiveVisitor<'de> {
    schema: &'de schema::Primitive,
}

impl<'de> PrimitiveVisitor<'de> {
    fn new(schema: &'de schema::Primitive) -> Self {
        Self { schema }
    }
}

impl<'de> de::Visitor<'de> for PrimitiveVisitor<'de> {
    type Value = value::Primitive;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.schema {
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

impl<'de> de::DeserializeSeed<'de> for PrimitiveVisitor<'de> {
    type Value = value::Primitive;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        match self.schema {
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

impl<'de> serde::de::Visitor<'de> for ValueVisitor<'de> {
    type Value = Value;

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
        Ok(Value::from_string(value.to_owned()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::from_string(v))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::from_bool(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::from_f64(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::from_u64(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::from_i64(v))
    }

    #[inline]
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        if let Schema::Object(fields) = self.schema {
            let mut rows = Vec::new();
            while let Some(field) = map.next_key_seed(FieldSelection::new(fields.as_slice()))? {
                match field {
                    Some(field) => {
                        let value_schema = field.schema;
                        match map.next_value_seed(Deserialize::new(&value_schema)) {
                            Ok(value) => rows.push((field.name.to_owned(), value)),
                            Err(err) => return Err(err),
                        };
                    }
                    None => {
                        let _: IgnoredAny = map.next_value()?;
                    }
                }
            }

            Ok(Value::Object(rows))
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

                while let Ok(Some(row)) = seq.next_element_seed(RowVisitor::new(map.as_slice())) {
                    rows.push(row.cols);
                }

                Ok(Value::Table { head: head.to_owned(), rows })
            }
            Schema::Array(primitive) => {
                let mut rows = Vec::with_capacity(seq.size_hint().unwrap_or(100));
                while let Ok(Some(row)) = seq.next_element_seed(PrimitiveVisitor::new(primitive)) {
                    rows.push(row);
                }

                Ok(Value::Array(rows))
            }
            _ => Err(de::Error::custom("expected a table or an array")),
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
