use serde::de::{self};
use serde_json::{self, Number};

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
                crate::schema::N::I8 => deserializer.deserialize_i8(visitor),
                crate::schema::N::I16 => deserializer.deserialize_i16(visitor),
                crate::schema::N::I32 => deserializer.deserialize_i32(visitor),
                crate::schema::N::I64 => deserializer.deserialize_i64(visitor),
                crate::schema::N::I128 => deserializer.deserialize_i128(visitor),
                crate::schema::N::U8 => deserializer.deserialize_u8(visitor),
                crate::schema::N::U16 => deserializer.deserialize_u16(visitor),
                crate::schema::N::U32 => deserializer.deserialize_u32(visitor),
                crate::schema::N::U64 => deserializer.deserialize_u64(visitor),
                crate::schema::N::U128 => deserializer.deserialize_u128(visitor),
                crate::schema::N::F32 => deserializer.deserialize_f32(visitor),
                crate::schema::N::F64 => deserializer.deserialize_f64(visitor),
            },
            Schema::String => deserializer.deserialize_str(visitor),
            Schema::Object(_) => deserializer.deserialize_map(visitor),
            Schema::Array(_) => deserializer.deserialize_seq(visitor),
        }
    }
}

pub struct Deserializer<'de> {
    input: &'de str,
}

impl<'de> Deserializer<'de> {
    pub fn new(input: &'de str) -> Self {
        Self { input }
    }

    fn invalid_type<V: de::Visitor<'de>>(&self, visitor: &V) -> serde_json::Result<V::Value> {
        Err(serde::de::Error::invalid_type(
            de::Unexpected::Str(&self.input),
            visitor,
        ))
    }
}

impl<'de> de::Deserializer<'de> for Deserializer<'de> {
    type Error = serde_json::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.input == "true" {
            visitor.visit_bool(true)
        } else if self.input == "false" {
            visitor.visit_bool(false)
        } else {
            self.invalid_type(&visitor)
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input.parse::<i32>() {
            Ok(n) => visitor.visit_i32(n),
            Err(_) => self.invalid_type(&visitor),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.input.parse::<f64>() {
            Ok(n) => visitor.visit_f64(n),
            Err(_) => self.invalid_type(&visitor),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.input.starts_with("\"") && self.input.ends_with("\"") {
            let len = self.input.len() - 1;
            visitor.visit_str(&self.input[1..len])
        } else {
            self.invalid_type(&visitor)
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        todo!()
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
                crate::schema::N::I8 => formatter.write_str("a i8"),
                crate::schema::N::I16 => formatter.write_str("a i16"),
                crate::schema::N::I32 => formatter.write_str("a i32"),
                crate::schema::N::I64 => formatter.write_str("a i64"),
                crate::schema::N::I128 => formatter.write_str("a i128"),
                crate::schema::N::U8 => formatter.write_str("a u8"),
                crate::schema::N::U16 => formatter.write_str("a u16"),
                crate::schema::N::U32 => formatter.write_str("a u32"),
                crate::schema::N::U64 => formatter.write_str("a u64"),
                crate::schema::N::U128 => formatter.write_str("a u128"),
                crate::schema::N::F32 => formatter.write_str("a f32"),
                crate::schema::N::F64 => formatter.write_str("a f64"),
            },
            Schema::Object(_) => formatter.write_str("a string"),
            Schema::Array(_) => formatter.write_str("a string"),
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

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(serde_json::Value::Number(Number::from(v)))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(serde_json::Value::Number(Number::from_f64(v).unwrap()))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(serde_json::Value::Null)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::schema::{Schema, N};

    fn check_schema(schema: Schema, input: &str) {
        let actual = schema.deserialize(input).unwrap();
        let expected = serde_json::from_str::<serde_json::Value>(input).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_string() {
        let schema = Schema::String;
        let input = r#""Hello World!""#;
        check_schema(schema, input);
    }

    #[test]
    fn test_bool() {
        let schema = Schema::Boolean;
        let input = r#"true"#;
        check_schema(schema, input);
    }

    #[test]
    fn test_i32() {
        let schema = Schema::Number(N::I32);
        let input = r#"42"#;
        check_schema(schema, input);
    }

    #[test]
    fn test_f64() {
        let schema = Schema::Number(N::F64);
        let input = r#"42.0"#;
        check_schema(schema, input);
    }

    fn test_number() {}
}
