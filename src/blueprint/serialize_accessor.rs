use std::{collections::hash_map::DefaultHasher, hash::{Hasher, Hash}};

use serde::{ser, Serialize};

pub struct SerializeAccessor {
  fields: Vec<String>,
  index: usize,
}

impl SerializeAccessor {
  pub fn new(fields: Vec<String>, index: usize) -> Self {
    Self { fields, index }
  }
}

pub struct SerializeAccessorHash {
  fields: Vec<String>,
  index: usize,
  hasher: DefaultHasher,
}

pub struct SerializeAccessorConcat {
  fields: Vec<String>,
  index: usize,
  result: String,
}

pub struct SerializeAccessorMap {
  fields: Vec<String>,
  index: usize,
  map_flag: bool,
  result: Option<String>,
}

#[derive(Debug)]
pub struct Error;

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self:?}")
  }
}

impl std::error::Error for Error {}

impl ser::Error for Error {
  fn custom<T>(msg: T) -> Self
  where
    T: std::fmt::Display,
  {
    Self
  }
}

type Result<T> = std::result::Result<T, Error>;

// pub fn to_string<T>(value: &T) -> Result<String>
// where
//     T: Serialize,
// {
//     let mut serializer = SerializeAccessor {
//         output: String::new(),
//     };
//     value.serialize(&mut serializer)?;
//     Ok(serializer.output)
// }

impl SerializeAccessor {
  fn serialize_to_string(self, v: impl ToString) -> Result<String> {
    Ok(v.to_string())
  }
}

impl SerializeAccessorConcat {
  fn end(mut self) -> Result<String> {
    self.result.push(')');
    Ok(self.result)
  }
}

impl SerializeAccessorMap {
  fn end(self) -> Result<String> {
    self.result.ok_or(Error)
  }
}

impl<'a> ser::Serializer for SerializeAccessor {
  type Ok = String;

  type Error = Error;

  type SerializeSeq = SerializeAccessorHash;
  type SerializeTuple = SerializeAccessorConcat;
  type SerializeTupleStruct = SerializeAccessorConcat;
  type SerializeTupleVariant = SerializeAccessorConcat;
  type SerializeMap = SerializeAccessorMap;
  type SerializeStruct = SerializeAccessorMap;
  type SerializeStructVariant = SerializeAccessorMap;

  fn serialize_bool(self, v: bool) -> Result<String> {
    self.serialize_to_string(v)
  }

  fn serialize_i8(self, v: i8) -> Result<String> {
    self.serialize_to_string(v)
  }

  fn serialize_i16(self, v: i16) -> Result<String> {
    self.serialize_to_string(v)
  }

  fn serialize_i32(self, v: i32) -> Result<String> {
    self.serialize_to_string(v)
  }

  fn serialize_i64(self, v: i64) -> Result<String> {
    self.serialize_to_string(v)
  }

  fn serialize_u8(self, v: u8) -> Result<String> {
    self.serialize_to_string(v)
  }

  fn serialize_u16(self, v: u16) -> Result<String> {
    self.serialize_to_string(v)
  }

  fn serialize_u32(self, v: u32) -> Result<String> {
    self.serialize_to_string(v)
  }

  fn serialize_u64(self, v: u64) -> Result<String> {
    self.serialize_to_string(v)
  }

  fn serialize_f32(self, v: f32) -> Result<String> {
    self.serialize_to_string(v)
  }

  fn serialize_f64(self, v: f64) -> Result<String> {
    self.serialize_to_string(v)
  }

  fn serialize_char(self, v: char) -> Result<String> {
    self.serialize_to_string(v)
  }

  fn serialize_str(self, v: &str) -> Result<String> {
    self.serialize_str(&v.to_string())
  }

  fn serialize_bytes(self, v: &[u8]) -> Result<String> {
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    Ok(hasher.finish().to_string())
  }

  fn serialize_none(self) -> Result<String> {
    Ok("".to_string())
  }

  fn serialize_some<T>(self, value: &T) -> Result<String>
  where
    T: ?Sized + Serialize,
  {
    value.serialize(self)
  }

  fn serialize_unit(self) -> Result<String> {
    Ok("null".to_string())
  }

  fn serialize_unit_struct(self, name: &'static str) -> Result<String> {
    self.serialize_unit()
  }

  fn serialize_unit_variant(self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<String> {
    self.serialize_str(variant)
  }

  fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<String>
  where
    T: ?Sized + Serialize,
  {
    value.serialize(self)
  }

  fn serialize_newtype_variant<T>(
    self,
    _name: &'static str,
    _variant_index: u32,
    variant: &'static str,
    value: &T,
  ) -> Result<String>
  where
    T: ?Sized + Serialize,
  {
    todo!()
    // self.output += "{";
    // variant.serialize(&mut *self)?;
    // self.output += ":";
    // value.serialize(&mut *self)?;
    // self.output += "}";
    // Ok(())
  }

  fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
    let Self { fields, index, .. } = self;
    let hasher = DefaultHasher::new();
    Ok(SerializeAccessorHash { fields, index, hasher })
  }

  fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
    let Self { fields, index, .. } = self;
    Ok(SerializeAccessorConcat { fields, index, result: "(".to_string() })
  }

  fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct> {
    self.serialize_tuple(len)
  }

  fn serialize_tuple_variant(
    self,
    _name: &'static str,
    _variant_index: u32,
    _variant: &'static str,
    len: usize,
  ) -> Result<Self::SerializeTupleVariant> {
    self.serialize_tuple(len)
  }

  fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
    let Self { fields, index } = self;
    Ok(SerializeAccessorMap { fields, index, map_flag: false, result: None })
  }

  fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
    self.serialize_map(Some(len))
  }

  fn serialize_struct_variant(
    self,
    _name: &'static str,
    _variant_index: u32,
    _variant: &'static str,
    len: usize,
  ) -> Result<Self::SerializeStructVariant> {
    self.serialize_map(Some(len))
  }
}

impl<'a> ser::SerializeSeq for SerializeAccessorHash {
  type Ok = String;
  type Error = Error;

  fn serialize_element<T>(&mut self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    value
      .serialize(SerializeAccessor { fields: self.fields.clone(), index: self.index })?
      .hash(&mut self.hasher);
    Ok(())
  }

  fn end(self) -> Result<String> {
    Ok(self.hasher.finish().to_string())
  }
}

impl<'a> ser::SerializeTuple for SerializeAccessorConcat {
  type Ok = String;
  type Error = Error;

  fn serialize_element<T>(&mut self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    if self.result.len() == 1 {
      self.result.push_str(", ");
    }
    self.result += &value
      .serialize(SerializeAccessor { fields: self.fields.clone(), index: self.index })?;
    Ok(())
  }

  fn end(self) -> Result<String> {
    self.end()
  }
}

impl<'a> ser::SerializeTupleStruct for SerializeAccessorConcat {
  type Ok = String;
  type Error = Error;

  fn serialize_field<T>(&mut self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    ser::SerializeTuple::serialize_element(self, value)
  }

  fn end(self) -> Result<String> {
    self.end()
  }
}

impl<'a> ser::SerializeTupleVariant for SerializeAccessorConcat {
  type Ok = String;
  type Error = Error;

  fn serialize_field<T>(&mut self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    ser::SerializeTuple::serialize_element(self, value)
  }

  fn end(self) -> Result<String> {
    self.end()
  }
}

impl<'a> ser::SerializeMap for SerializeAccessorMap {
  type Ok = String;
  type Error = Error;

  fn serialize_key<T>(&mut self, key: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    let key = key
      .serialize(SerializeAccessor { fields: self.fields.clone(), index: self.index })?;
    self.map_flag = &key == self.fields.get(self.index).ok_or(Error)?;
    Ok(())
  }

  fn serialize_value<T>(&mut self, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    if self.map_flag {
      self.result = Some(value.serialize(SerializeAccessor { fields: self.fields.clone(), index: self.index })?);
    }
    Ok(())
  }

  fn end(self) -> Result<String> {
    self.end()
  }
}

impl<'a> ser::SerializeStruct for SerializeAccessorMap {
  type Ok = String;
  type Error = Error;

  fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    let cur = self.fields.get(self.index).ok_or(Error)?;
    if key == cur.as_str() {
      self.index += 1;
      self.result = Some(value.serialize(SerializeAccessor { fields: self.fields.clone(), index: self.index })?);
    }
    Ok(())
  }

  fn end(self) -> Result<String> {
    self.end()
  }
}

impl<'a> ser::SerializeStructVariant for SerializeAccessorMap {
  type Ok = String;
  type Error = Error;

  fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
  where
    T: ?Sized + Serialize,
  {
    ser::SerializeStruct::serialize_field(self, key, value)
  }

  fn end(self) -> Result<String> {
    self.end()
  }
}
