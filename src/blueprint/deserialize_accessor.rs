use std::any::Any;
use std::cell::RefCell;
use std::ops::{AddAssign, MulAssign, Neg};
use std::rc::Rc;
use std::str::Utf8Error;
use std::fmt;

use serde::Deserialize;
use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor, DeserializeOwned,
};
use serde_json::de::SliceRead;

#[derive(Debug, Clone)]
enum Error {
    TrailingCharacters,
    Eof,
    ExpectedBoolean,
    ExpectedUnsignedInteger,
    ExpectedString,
    ExpectedArrayComma,
    ExpectedMapComma,
    Custom(String),
    Utf8Err(Utf8Error),
    Syntax,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl de::Error for Error {
    fn custom<T>(msg:T) -> Self where T:std::fmt::Display {
        Error::Custom(format!("{}", msg))
    }
}

// pub struct DeserializeAccessorOwned<'de> {
//     fields: Vec<String>,
//     index: usize,
// }

pub struct DeserializeAccessor {
    pub result: String
}

pub struct DeserializerAccessor<'de> {
    fields: &'de Vec<String>,
    index: usize,
    inner_deserializer: serde_json::Deserializer<SliceRead<'de>>,
}

impl<'de> DeserializerAccessor<'de> {
    pub fn from_bytes(input: &'de [u8], fields: &'de Vec<String>) -> DeserializerAccessor<'de> {
        let slice_read = SliceRead::new(input);
        let inner_deserializer = serde_json::Deserializer::new(slice_read);
        DeserializerAccessor { inner_deserializer, fields, index: 0 }
    }

    fn unsafe_modify_visitor(&self, visitor: &dyn Any) {
        if let Some(accessor_visitor) = visitor.downcast_ref::<AccessorVisitor<'de>>() {
            *accessor_visitor.fields.borrow_mut() = Some(self.fields);
        }
    }
}

impl<'de> Deserialize<'de> for DeserializeAccessor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(AccessorVisitor::empty())
    }
}

impl<'de, 'a> de::Deserializer<'de> for DeserializerAccessor<'de> {
    type Error = serde_json::error::Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.unsafe_modify_visitor(&visitor);
        self.inner_deserializer.deserialize_any(visitor)
    }

    fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_bool(visitor)
    }

    fn deserialize_i8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_i8(visitor)
    }

    fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_i16(visitor)
    }

    fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_i32(visitor)
    }

    fn deserialize_i64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_i64(visitor)
    }

    fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_u8(visitor)
    }

    fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_u16(visitor)
    }

    fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_u32(visitor)
    }

    fn deserialize_u64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_u64(visitor)
    }

    fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_f32(visitor)
    }

    fn deserialize_f64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_f32(visitor)
    }

    fn deserialize_char<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_f32(visitor)
    }

    fn deserialize_str<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_f32(visitor)
    }

    fn deserialize_string<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_f32(visitor)
    }

    fn deserialize_bytes<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_f32(visitor)
    }

    fn deserialize_byte_buf<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_f32(visitor)
    }

    fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_f32(visitor)
    }

    fn deserialize_unit<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_unit(visitor)
    }

    fn deserialize_unit_struct<V>(
        mut self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_unit_struct(name, visitor)
    }

    fn deserialize_newtype_struct<V>(
        mut self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_newtype_struct(name, visitor)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_seq(visitor)
    }

    fn deserialize_tuple<V>(mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_tuple(len, visitor)
    }

    fn deserialize_tuple_struct<V>(
        mut self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_tuple_struct(name, len, visitor)
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_map(visitor)
    }

    fn deserialize_struct<V>(
        mut self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_struct(name, fields, visitor)
    }

    fn deserialize_enum<V>(
        mut self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_enum(name, variants, visitor)
    }

    fn deserialize_identifier<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_identifier(visitor)
    }

    fn deserialize_ignored_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.inner_deserializer.deserialize_any(visitor)
    }
}

// struct CommaSeparated<'a, 'de: 'a> {
//     de: &'a mut DeserializeAccessor<'de>,
//     first: bool,
//     found: bool,
// }

// impl<'a, 'de> CommaSeparated<'a, 'de> {
//     fn new(de: &'a mut DeserializeAccessor<'de>) -> Self {
//         CommaSeparated {
//             de,
//             first: true,
//             found: false
//         }
//     }
// }

// impl<'de, 'a> SeqAccess<'de> for CommaSeparated<'a, 'de> {
//     type Error = Error;

//     fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
//     where
//         T: DeserializeSeed<'de>,
//     {
//         if self.de.peek_char()? == ']' {
//             return Ok(None);
//         }
//         if !self.first && self.de.next_char()? != ',' {
//             return Err(Error::ExpectedArrayComma);
//         }
//         self.first = false;
//         seed
//             .deserialize(&mut *self.de)
//             .map(|key| {
                
//                 Some(key)
//             })
//     }
// }

// impl<'de, 'a> MapAccess<'de> for CommaSeparated<'a, 'de> {
//     type Error = Error;

//     fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
//     where
//         K: DeserializeSeed<'de>,
//     {
//         if self.de.peek_char()? == '}' {
//             return Ok(None);
//         }
//         if !self.first && self.de.next_char()? != ',' {
//             return Err(Error::ExpectedMapComma);
//         }
//         self.first = false;
//         seed.deserialize(&mut *self.de).map(Some)
//     }

//     fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
//     where
//         V: DeserializeSeed<'de>,
//     {
//         if self.de.next_char()? != ':' {
//             return Err(Error::ExpectedMapColon);
//         }
//         seed.deserialize(&mut *self.de)
//     }
// }

// struct Enum<'a, 'de: 'a> {
//     de: &'a mut DeserializeAccessor<'de>,
// }

// impl<'a, 'de> Enum<'a, 'de> {
//     fn new(de: &'a mut DeserializeAccessor<'de>) -> Self {
//         Enum { de }
//     }
// }

// impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
//     type Error = Error;
//     type Variant = Self;

//     fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
//     where
//         V: DeserializeSeed<'de>,
//     {
//         let val = seed.deserialize(&mut *self.de)?;
//         if self.de.next_char()? == ':' {
//             Ok((val, self))
//         } else {
//             Err(Error::ExpectedMapColon)
//         }
//     }
// }

// impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
//     type Error = Error;

//     fn unit_variant(self) -> Result<()> {
//         Err(Error::ExpectedString)
//     }

//     fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
//     where
//         T: DeserializeSeed<'de>,
//     {
//         seed.deserialize(self.de)
//     }

//     fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         de::Deserializer::deserialize_seq(self.de, visitor)
//     }

//     fn struct_variant<V>(
//         self,
//         _fields: &'static [&'static str],
//         visitor: V,
//     ) -> Result<V::Value>
//     where
//         V: Visitor<'de>,
//     {
//         de::Deserializer::deserialize_map(self.de, visitor)
//     }
// }

struct AccessorVisitor<'de> {
    fields: Rc<RefCell<Option<&'de Vec<String>>>>,
    index: usize
}

impl<'de> AccessorVisitor<'de> {
    fn empty() -> Self {
        Self { fields: Rc::new(RefCell::new(None)), index: 0 }
    }
}

// struct AccessorVisitor<'de> {
//     fields: &'de Vec<String>,
//     index: usize,
// }

// trait AccessorVisitorCreator<'de> {
//     fn new(&'de self) -> AccessorVisitor<'de> {
//         unimplemented!()
//     }
// }

// impl<'de> AccessorVisitorCreator<'de> for DeserializerAccessor<'de> {
//     fn new(&'de self) -> AccessorVisitor<'de> {
//         AccessorVisitor { fields: self.fields, index: self.index }
//     }
// }

// impl<'de> AccessorVisitor<'de> {
//     fn visit_value<E, T: ToString>(self, value: T) -> std::result::Result<String, E> {
//         Ok(value.to_string())
//     }

    // fn deserializer(&self) -> DeserializeAccessor<'de> {
    //     DeserializeAccessor { inner_deserializer }
    // }
// }

impl<'de> Visitor<'de> for AccessorVisitor<'de> {
    type Value = DeserializeAccessor;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between -2^31 and 2^31")
    }

    fn visit_map<A>(self, mut map: A) -> std::prelude::v1::Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>, {
        loop {
            if let Some(key) = map.next_key::<String>()? {
                if Some(&key) == self.fields.borrow().and_then(|fields| fields.get(self.index)) {
                    if self.index + 1 == self.fields.borrow().unwrap().len() {
                        return Ok(DeserializeAccessor { result: map.next_value::<String>()? });
                    }
                    return map.next_value();
                }
            }
        }
    }
}