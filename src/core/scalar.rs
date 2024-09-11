use std::collections::HashMap;
use std::fmt::Debug;

use lazy_static::lazy_static;
use schemars::schema::{InstanceType, Schema, SchemaObject};
use strum::IntoEnumIterator;
use tailcall_macros::{gen_doc, Doc};

use crate::core::json::JsonLike;

const PREDEFINED_SCALARS: &[&str] = &["Boolean", "Float", "ID", "Int", "String"];

lazy_static! {
    static ref CUSTOM_SCALARS: HashMap<String, Scalar> =
        Scalar::iter().map(|v| (v.name(), v)).collect();
}

#[derive(
    schemars::JsonSchema, Debug, Clone, strum_macros::Display, strum_macros::EnumIter, Doc,
)]
pub enum Scalar {
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
    /// Field whose value conforms to the standard datetime format as specified in RFC 3339 (https://datatracker.ietf.org/doc/html/rfc3339").
    #[gen_doc(ty = "String")]
    DateTime,
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

fn eval_str<'a, Value: JsonLike<'a>, F: Fn(&str) -> bool>(val: &'a Value, fxn: F) -> bool {
    val.as_str().map_or(false, fxn)
}

fn eval_signed<
    'a,
    Num,
    Value: JsonLike<'a>,
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
    Value: JsonLike<'a>,
    F: Fn(u64) -> Result<Num, std::num::TryFromIntError>,
>(
    val: &'a Value,
    fxn: F,
) -> bool {
    val.as_u64().map_or(false, |n| fxn(n).is_ok())
}

impl Scalar {
    ///
    /// Check if the type is a predefined scalar
    pub fn is_predefined(type_name: &str) -> bool {
        if PREDEFINED_SCALARS.iter().any(|v| type_name.eq(*v)) {
            true
        } else {
            CUSTOM_SCALARS.get(type_name).is_some()
        }
    }

    pub fn validate<'a, Value: JsonLike<'a>>(&self, value: &'a Value) -> bool {
        match self {
            Scalar::JSON => true,
            Scalar::Empty => true,
            Scalar::Email => eval_str(value, |s| {
                async_graphql::validators::email(&s.to_string()).is_ok()
            }),
            Scalar::PhoneNumber => eval_str(value, |s| phonenumber::parse(None, s).is_ok()),
            Scalar::Date => eval_str(value, |s| chrono::DateTime::parse_from_rfc3339(s).is_ok()),
            Scalar::DateTime => {
                eval_str(value, |s| chrono::DateTime::parse_from_rfc3339(s).is_ok())
            }
            Scalar::Url => eval_str(value, |s| url::Url::parse(s).is_ok()),
            Scalar::Bytes => value.as_str().is_some(),

            Scalar::Int64 => eval_str(value, |s| s.parse::<i64>().is_ok()),
            Scalar::UInt64 => eval_str(value, |s| s.parse::<u64>().is_ok()),
            Scalar::Int128 => eval_str(value, |s| s.parse::<i128>().is_ok()),
            Scalar::UInt128 => eval_str(value, |s| s.parse::<u128>().is_ok()),

            Scalar::Int8 => eval_signed(value, i8::try_from),
            Scalar::Int16 => eval_signed(value, i16::try_from),
            Scalar::Int32 => eval_signed(value, i32::try_from),

            Scalar::UInt8 => eval_unsigned(value, u8::try_from),
            Scalar::UInt16 => eval_unsigned(value, u16::try_from),
            Scalar::UInt32 => eval_unsigned(value, u32::try_from),
        }
    }
    pub fn find(name: &str) -> Option<&Scalar> {
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
    use async_graphql_value::ConstValue;
    use schemars::schema::Schema;

    use crate::core::scalar::{Scalar, CUSTOM_SCALARS};

    /// generates test asserts for valid scalar inputs
    #[macro_export]
    macro_rules! test_scalar_valid {
    ($instance: expr, $($value: expr),+) => {
        #[test]
        fn test_scalar_valid() {
            let value = $instance;

            $(
                assert!(value.validate::<async_graphql_value::ConstValue>(&$value));
            )+
        }
    };
}

    // generates test asserts for invalid scalar inputs
    #[macro_export]
    macro_rules! test_scalar_invalid {
    ($instance: expr, $($value: expr),+) => {
        #[test]
        fn test_scalar_invalid() {
            let value = $instance;

            $(
                assert!(!value.validate::<async_graphql_value::ConstValue>(&$value));
            )+
        }
    };
}

    mod bytes {
        use serde_json::Number;

        use super::{ConstValue, Scalar};

        test_scalar_valid! {
            Scalar::Bytes,
            ConstValue::String("\0\0".to_string())
        }
        test_scalar_invalid! {
            Scalar::Bytes,
            ConstValue::Null,
            ConstValue::Number(Number::from_f64(1.25).unwrap())
        }
    }

    mod date {
        use super::{ConstValue, Scalar};
        test_scalar_valid! {
            Scalar::Date,
            ConstValue::String("2020-01-01T12:00:00Z".to_string())
        }
        test_scalar_invalid! {
            Scalar::Date,
            ConstValue::String("2023-03-08T12:45:26".to_string()),
            ConstValue::Null
        }
    }
    mod email {
        use super::{ConstValue, Scalar};
        test_scalar_valid! {
            Scalar::Email,
            ConstValue::String("valid@email.com".to_string())
        }
        test_scalar_invalid! {
            Scalar::Email,
            ConstValue::String("invalid_email".to_string()),
            ConstValue::Null
        }
    }

    mod i128 {
        use serde_json::Number;

        use super::{ConstValue, Scalar};
        test_scalar_valid! {
            Scalar::Int128,
            ConstValue::String("100".to_string()),
            ConstValue::String("-15".to_string()),
            ConstValue::String(i128::MAX.to_string())
        }

        test_scalar_invalid! {
            Scalar::Int128,
            ConstValue::Null,
            ConstValue::Number(Number::from(15)),
            ConstValue::Number(
                Number::from_f64(1.25).unwrap()
            ),
            ConstValue::String(format!("{}0", i128::MAX))
        }
    }

    mod i16 {
        use serde_json::Number;

        use super::{ConstValue, Scalar};

        test_scalar_valid! {
            Scalar::Int16,
            ConstValue::Number(Number::from(100u32)),
            ConstValue::Number(Number::from(2 * i8::MAX as i64)),
            ConstValue::Number(
                Number::from(-15)
            )
        }

        test_scalar_invalid! {
            Scalar::Int16,
            ConstValue::Null,
            ConstValue::Number(Number::from(i16::MAX as i64 + 1)),
            ConstValue::Number(Number::from(i16::MIN as i64 - 1)),
            ConstValue::Number(
                Number::from_f64(1.25).unwrap()
            ),
            ConstValue::String("4564846".to_string())
        }
    }

    mod i32 {
        use serde_json::Number;

        use super::{ConstValue, Scalar};

        test_scalar_valid! {
            Scalar::Int32,
            ConstValue::Number(Number::from(100u32)),
            ConstValue::Number(Number::from(i32::MAX as i64)),
            ConstValue::Number(
                Number::from(-15)
            )
        }

        test_scalar_invalid! {
            Scalar::Int32,
            ConstValue::Null,
            ConstValue::Number(Number::from(i32::MAX as i64 + 1)),
            ConstValue::Number(Number::from(i32::MIN as i64 - 1)),
            ConstValue::Number(
                Number::from_f64(1.25).unwrap()
            ),
            ConstValue::String("4564846".to_string())
        }
    }

    mod i64 {
        use serde_json::Number;

        use super::{ConstValue, Scalar};

        test_scalar_valid! {
            Scalar::Int64,
            ConstValue::String("125".to_string()),
            ConstValue::String("-15".to_string()),
            ConstValue::String(i64::MAX.to_string())
        }

        test_scalar_invalid! {
            Scalar::Int64,
            ConstValue::Null,
            ConstValue::Number(Number::from(15)),
            ConstValue::Number(
                Number::from_f64(1.25).unwrap()
            ),
            ConstValue::String(format!("{}1", i64::MAX))
        }
    }

    mod i8 {
        use serde_json::Number;

        use super::{ConstValue, Scalar};

        test_scalar_valid! {
            Scalar::Int8,
            ConstValue::Number(Number::from(100i32)),
            ConstValue::Number(Number::from(127)),
            ConstValue::Number(
                Number::from(-15)
            )
        }

        test_scalar_invalid! {
            Scalar::Int8,
            ConstValue::Null,
            ConstValue::Number(Number::from(128)),
            ConstValue::Number(Number::from(-129)),
            ConstValue::Number(
                Number::from_f64(1.25).unwrap()
            ),
            ConstValue::String("4564846".to_string())
        }
    }

    mod phone {
        use super::{ConstValue, Scalar};

        test_scalar_valid! {
            Scalar::PhoneNumber,
            ConstValue::String("+911234567890".to_string())
        }

        test_scalar_invalid! {
            Scalar::PhoneNumber,
            ConstValue::String("1234567890".to_string()),
            ConstValue::Null
        }
    }

    mod u128 {
        use serde_json::Number;

        use super::{ConstValue, Scalar};

        test_scalar_valid! {
            Scalar::UInt128,
            ConstValue::String("100".to_string()),
            ConstValue::String(u128::MAX.to_string())
        }

        test_scalar_invalid! {
            Scalar::UInt128,
            ConstValue::Null,
            ConstValue::Number(Number::from(15)),
            ConstValue::Number(
                Number::from_f64(1.25).unwrap()
            ),
            ConstValue::String("-1".to_string()),
            ConstValue::String(format!("{}0", u128::MAX))
        }
    }

    mod u16 {
        use serde_json::Number;

        use super::{ConstValue, Scalar};

        test_scalar_valid! {
            Scalar::UInt16,
            ConstValue::Number(Number::from(100u32)),
            ConstValue::Number(Number::from(2 * u8::MAX as u64))
        }

        test_scalar_invalid! {
           Scalar::UInt16,
            ConstValue::Null,
            ConstValue::Number(Number::from(u16::MAX as u64 + 1)),
            ConstValue::Number(Number::from(-1)),
            ConstValue::Number(
                Number::from_f64(1.25).unwrap()
            ),
            ConstValue::String("4564846".to_string())
        }
    }

    mod u32 {
        use serde_json::Number;

        use super::{ConstValue, Scalar};

        test_scalar_valid! {
            Scalar::UInt32,
            ConstValue::Number(Number::from(100u32)),
            ConstValue::Number(Number::from(u32::MAX as u64))
        }

        test_scalar_invalid! {
            Scalar::UInt32,
            ConstValue::Null,
            ConstValue::Number(Number::from(u32::MAX as u64 + 1)),
            ConstValue::Number(Number::from(-1)),
            ConstValue::Number(
                Number::from_f64(1.25).unwrap()
            ),
            ConstValue::String("4564846".to_string())
        }
    }

    mod u64 {
        use serde_json::Number;

        use super::{ConstValue, Scalar};

        test_scalar_valid! {
            Scalar::UInt64,
            ConstValue::String("125".to_string()),
            ConstValue::String(u64::MAX.to_string())
        }

        test_scalar_invalid! {
            Scalar::UInt64,
            ConstValue::Null,
            ConstValue::Number(Number::from(15)),
            ConstValue::Number(
                Number::from_f64(1.25).unwrap()
            ),
            ConstValue::String("-1".to_string()),
            ConstValue::String(format!("{}1", u64::MAX))
        }
    }

    mod u8 {
        use serde_json::Number;

        use super::{ConstValue, Scalar};

        test_scalar_valid! {
            Scalar::UInt8,
            ConstValue::Number(Number::from(15)),
            ConstValue::Number(Number::from(255))
        }

        test_scalar_invalid! {
            Scalar::UInt8,
            ConstValue::Null,
            ConstValue::Number(Number::from(256)),
            ConstValue::Number(Number::from(-1)),
            ConstValue::Number(
                Number::from_f64(1.25).unwrap()
            ),
            ConstValue::String("4564846".to_string())
        }
    }

    mod url {
        use super::{ConstValue, Scalar};

        test_scalar_valid! {
            Scalar::Url,
            ConstValue::String("https://ssdd.dev".to_string())
        }

        test_scalar_invalid! {
            Scalar::Url,
            ConstValue::Null,
            ConstValue::String("localhost".to_string())
        }
    }

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
