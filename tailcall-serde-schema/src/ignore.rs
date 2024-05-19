use serde::de::{self};

pub struct Ignore;

impl<'de> de::Visitor<'de> for Ignore {
    type Value = Ignore;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("anything at all")
    }

    fn visit_bool<E>(self, _: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_i8<E>(self, _: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_i16<E>(self, _: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_i32<E>(self, _: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_i128<E>(self, _: i128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_u8<E>(self, _: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_u16<E>(self, _: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_u32<E>(self, _: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_u128<E>(self, _: u128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_f32<E>(self, _: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_f64<E>(self, _: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_char<E>(self, _: char) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_str<E>(self, _: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_borrowed_str<E>(self, _: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_string<E>(self, _: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_bytes<E>(self, _: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_borrowed_bytes<E>(self, _: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_byte_buf<E>(self, _: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_some<D>(self, _: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Ok(Ignore)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Ignore)
    }

    fn visit_newtype_struct<D>(self, _: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Ok(Ignore)
    }

    fn visit_seq<A>(self, _: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        Ok(Ignore)
    }

    fn visit_map<A>(self, _: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        Ok(Ignore)
    }

    fn visit_enum<A>(self, _: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        Ok(Ignore)
    }
}

impl<'de> de::DeserializeSeed<'de> for Ignore {
    type Value = Ignore;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_ignored_any(Ignore)
    }
}
