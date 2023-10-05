use std::collections::BTreeMap;

pub use derive_schema::Schema;
pub trait Schema {
  fn schema() -> DynamicSchema;
}

#[derive(Debug, PartialEq)]
pub enum DynamicSchema {
  String,
  Number(Number),
  Boolean,
  Record {
    name: String,
    fields: BTreeMap<String, DynamicSchema>,
  },
  Enum(Vec<DynamicSchema>),
  Unit
}

impl DynamicSchema {
  pub fn record(name: &'static str) -> DynamicSchema {
    Self::Record { name: name.to_owned(), fields: BTreeMap::new() }
  }
}

#[derive(Debug, PartialEq)]
pub enum Number {
  I8,
  I16,
  I32,
  I64,
  U8,
  U16,
  U32,
  U64,
  F32,
  F64,
}

impl Schema for String {
  fn schema() -> DynamicSchema {
    DynamicSchema::String
  }
}

impl Schema for i8 {
  fn schema() -> DynamicSchema {
    DynamicSchema::Number(Number::I8)
  }
}
impl Schema for i16 {
  fn schema() -> DynamicSchema {
    DynamicSchema::Number(Number::I16)
  }
}
impl Schema for i32 {
  fn schema() -> DynamicSchema {
    DynamicSchema::Number(Number::I32)
  }
}
impl Schema for i64 {
  fn schema() -> DynamicSchema {
    DynamicSchema::Number(Number::I64)
  }
}
impl Schema for u8 {
  fn schema() -> DynamicSchema {
    DynamicSchema::Number(Number::U8)
  }
}
impl Schema for u16 {
  fn schema() -> DynamicSchema {
    DynamicSchema::Number(Number::U16)
  }
}
impl Schema for u32 {
  fn schema() -> DynamicSchema {
    DynamicSchema::Number(Number::U32)
  }
}
impl Schema for u64 {
  fn schema() -> DynamicSchema {
    DynamicSchema::Number(Number::U64)
  }
}
impl Schema for f32 {
  fn schema() -> DynamicSchema {
    DynamicSchema::Number(Number::F32)
  }
}
impl Schema for f64 {
  fn schema() -> DynamicSchema {
    DynamicSchema::Number(Number::F64)
  }
}

impl Schema for bool {
  fn schema() -> DynamicSchema {
    DynamicSchema::Boolean
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, PartialEq, Schema)]
  struct Person {
    name: String,
    age: u32,
  }

  // #[derive(Debug, PartialEq, Schema)]
  // enum Color {
  //   Red,
  //   Green,
  //   Blue,
  // }

  #[test]
  fn test_derive_schema_for_person() {
    let expected_schema = DynamicSchema::Record {
      name: "Person".to_string(),
      fields: {
        let mut fields = BTreeMap::new();
        fields.insert("name".to_owned(), DynamicSchema::String);
        fields.insert("age".to_owned(), DynamicSchema::Number(Number::U32));
        fields
      },
    };

    assert_eq!(Person::schema(), expected_schema);
  }

  // #[test]
  // fn test_derive_schema_for_color() {
  //   let expected_schema = DynamicSchema::Enum(vec![
  //     DynamicSchema::record("Red"),
  //     DynamicSchema::record("Green"),
  //     DynamicSchema::record("Blue"),
  //   ]);

  //   assert_eq!(Color::schema(), expected_schema);
  // }
}
