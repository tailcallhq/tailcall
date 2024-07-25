use std::collections::HashMap;
use std::fmt::Debug;

use lazy_static::lazy_static;
use schemars::schema::{InstanceType, Schema, SchemaObject};
use strum::IntoEnumIterator;
use tailcall_macros::{gen_doc, Doc};

use crate::core::json::JsonLike;

lazy_static! {
    static ref CUSTOM_SCALARS: HashMap<String, ScalarType> =
        ScalarType::iter().map(|v| (v.name(), v)).collect();
}

#[derive(
    schemars::JsonSchema, Debug, Clone, strum_macros::Display, strum_macros::EnumIter, Doc,
)]
pub enum ScalarType {
    /// Empty scalar type represents an empty value.
    #[gen_doc(ty = "Null")]
    Empty,
    /// Field whose value conforms to the standard internet email address format as specified in HTML Spec: https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address.
    #[gen_doc(ty = "String")]
    Email,
    /// Field whose value conforms to the standard E.164 format as specified in E.164 specification (https://en.wikipedia.org/wiki/E.164).
    #[gen_doc(ty = "String")]
    PhoneNumber,
    /// Field whose value conforms to the standard date format as specified in RFC 3339 (https://datatracker.ietf.org/doc/html/rfc3339).
    #[gen_doc(ty = "String")]
    Date,
    /// Field whose value conforms to the standard URL format as specified in RFC 3986 (https://datatracker.ietf.org/doc/html/rfc3986).
    #[gen_doc(ty = "String")]
    Url,
    /// Field whose value conforms to the standard JSON format as specified in RFC 8259 (https://datatracker.ietf.org/doc/html/rfc8259).
    #[gen_doc(ty = "Object")]
    JSON,
    /// Field whose value is an 8-bit signed integer.
    #[gen_doc(ty = "Integer")]
    Int8,
    /// Field whose value is a 16-bit signed integer.
    #[gen_doc(ty = "Integer")]
    Int16,
    /// Field whose value is a 32-bit signed integer.
    #[gen_doc(ty = "Integer")]
    Int32,
    /// Field whose value is a 64-bit signed integer.
    #[gen_doc(ty = "Integer")]
    Int64,
    /// Field whose value is a 128-bit signed integer.
    #[gen_doc(ty = "Integer")]
    Int128,
    /// Field whose value is an 8-bit unsigned integer.
    #[gen_doc(ty = "Integer")]
    UInt8,
    /// Field whose value is a 16-bit unsigned integer.
    #[gen_doc(ty = "Integer")]
    UInt16,
    /// Field whose value is a 32-bit unsigned integer.
    #[gen_doc(ty = "Integer")]
    UInt32,
    /// Field whose value is a 64-bit unsigned integer.
    #[gen_doc(ty = "Integer")]
    UInt64,
    /// Field whose value is a 128-bit unsigned integer.
    #[gen_doc(ty = "Integer")]
    UInt128,
    /// Field whose value is a sequence of bytes.
    #[gen_doc(ty = "String")]
    Bytes,
}

fn eval_str<'a, Value: JsonLike<'a> + 'a, F: Fn(&str) -> bool>(val: &'a Value, fxn: F) -> bool {
    val.as_str().map_or(false, fxn)
}

fn eval_signed<
    'a,
    Num,
    Value: JsonLike<'a> + 'a,
    F: Fn(i64) -> Result<Num, std::num::TryFromIntError>,
>(
    val: &'a Value,
    fxn: F,
) -> bool {
    val.as_i64().map_or(false, |n| fxn(n).is_ok())
}

fn eval_unsigned<
    'a,
    Num,
    Value: JsonLike<'a> + 'a,
    F: Fn(u64) -> Result<Num, std::num::TryFromIntError>,
>(
    val: &'a Value,
    fxn: F,
) -> bool {
    val.as_u64().map_or(false, |n| fxn(n).is_ok())
}

impl ScalarType {
    ///
    /// Check if the type is a predefined scalar
    pub fn is_predefined_scalar(type_name: &str) -> bool {
        let predefined = ["String", "Int", "Float", "ID", "Boolean"];
        if predefined.iter().any(|v| type_name.eq(*v)) {
            true
        } else {
            CUSTOM_SCALARS.get(type_name).is_some()
        }
    }

    pub fn validate<'a, Value: JsonLike<'a> + 'a>(&self, value: &'a Value) -> bool {
        match self {
            ScalarType::JSON => true,
            ScalarType::Empty => true,
            ScalarType::Email => eval_str(value, |s| {
                async_graphql::validators::email(&s.to_string()).is_ok()
            }),
            ScalarType::PhoneNumber => eval_str(value, |s| phonenumber::parse(None, s).is_ok()),
            ScalarType::Date => {
                eval_str(value, |s| chrono::DateTime::parse_from_rfc3339(s).is_ok())
            }
            ScalarType::Url => eval_str(value, |s| url::Url::parse(s).is_ok()),
            ScalarType::Bytes => value.as_str().is_some(),

            ScalarType::Int64 => eval_str(value, |s| s.parse::<i64>().is_ok()),
            ScalarType::UInt64 => eval_str(value, |s| s.parse::<u64>().is_ok()),
            ScalarType::Int128 => eval_str(value, |s| s.parse::<i128>().is_ok()),
            ScalarType::UInt128 => eval_str(value, |s| s.parse::<u128>().is_ok()),

            ScalarType::Int8 => eval_signed(value, i8::try_from),
            ScalarType::Int16 => eval_signed(value, i16::try_from),
            ScalarType::Int32 => eval_signed(value, i32::try_from),

            ScalarType::UInt8 => eval_unsigned(value, u8::try_from),
            ScalarType::UInt16 => eval_unsigned(value, u16::try_from),
            ScalarType::UInt32 => eval_unsigned(value, u32::try_from),
        }
    }
    pub fn scalar(name: &str) -> Option<&ScalarType> {
        CUSTOM_SCALARS.get(name)
    }
    pub fn name(&self) -> String {
        self.to_string()
    }
    pub fn scalar_definition(&self) -> async_graphql::parser::types::TypeSystemDefinition {
        let schemars = self.schema();
        tailcall_typedefs_common::scalar_definition::into_scalar_definition(schemars, &self.name())
    }
    pub fn schema(&self) -> Schema {
        let type_of = self.ty();
        let format = match type_of {
            InstanceType::Integer => Some(self.name().to_lowercase()),
            _ => None,
        };
        let mut value = serde_json::json!(
            {
                "title": self.name(),
                "type": type_of,
                "description": self.doc(),
            }
        );
        if let Some(format) = format {
            value["format"] = serde_json::json!(format);
        }

        let metadata = serde_json::from_value(value).unwrap();
        Schema::Object(SchemaObject { metadata: Some(Box::new(metadata)), ..Default::default() })
    }
}

#[cfg(test)]
mod test {
    use schemars::schema::Schema;

    use crate::core::scalar::CUSTOM_SCALARS;

    fn get_name(v: Schema) -> String {
        serde_json::to_value(v)
            .unwrap()
            .as_object()
            .unwrap()
            .get("title")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn assert_scalar_types() {
        // it's easy to accidentally add a different scalar type to the schema
        // this test ensures that the scalar types are correctly defined
        for (k, v) in CUSTOM_SCALARS.iter() {
            assert_eq!(k.clone(), get_name(v.schema()));
        }
    }
}
