use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use lazy_static::lazy_static;
use schemars::schema::{InstanceType, Schema, SchemaObject};

use crate::core::json::JsonLike;

#[derive(schemars::JsonSchema, Debug, Clone, strum_macros::Display)]
pub enum ScalarType {
    Empty,
    Email,
    PhoneNumber,
    Date,
    Url,
    JSON,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
    Bytes,
}

fn validate_str<'a, Value: JsonLike<'a> + 'a, F: Fn(&str) -> bool>(val: &'a Value, fxn: F) -> bool {
    val.as_str().map_or(false, fxn)
}

fn validate_signed<
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

fn validate_unsigned<
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
    pub fn validate<'a, Value: JsonLike<'a> + 'a>(&self, value: &'a Value) -> bool {
        match self {
            ScalarType::JSON => true,
            ScalarType::Empty => true,
            ScalarType::Email => validate_str(value, |s| {
                async_graphql::validators::email(&s.to_string()).is_ok()
            }),
            ScalarType::PhoneNumber => validate_str(value, |s| phonenumber::parse(None, s).is_ok()),
            ScalarType::Date => {
                validate_str(value, |s| chrono::DateTime::parse_from_rfc3339(s).is_ok())
            }
            ScalarType::Url => validate_str(value, |s| url::Url::parse(s).is_ok()),
            ScalarType::Bytes => value.as_str().is_some(),

            ScalarType::Int64 => validate_str(value, |s| s.parse::<i64>().is_ok()),
            ScalarType::UInt64 => validate_str(value, |s| s.parse::<u64>().is_ok()),
            ScalarType::Int128 => validate_str(value, |s| s.parse::<i128>().is_ok()),
            ScalarType::UInt128 => validate_str(value, |s| s.parse::<u128>().is_ok()),

            ScalarType::Int8 => validate_signed(value, i8::try_from),
            ScalarType::Int16 => validate_signed(value, i16::try_from),
            ScalarType::Int32 => validate_signed(value, i32::try_from),

            ScalarType::UInt8 => validate_unsigned(value, u8::try_from),
            ScalarType::UInt16 => validate_unsigned(value, u16::try_from),
            ScalarType::UInt32 => validate_unsigned(value, u32::try_from),
        }
    }
    pub fn get_scalar(name: &str) -> ScalarType {
        CUSTOM_SCALARS.get(name).cloned().unwrap_or(Self::Empty)
    }
    pub fn name(&self) -> String {
        self.to_string()
    }
    pub fn scalar_definition(&self) -> async_graphql::parser::types::TypeSystemDefinition {
        let schemars = self.schema();
        tailcall_typedefs_common::scalar_definition::into_scalar_definition(schemars, &self.name())
    }
    fn schema_inner(&self, type_of: InstanceType, description: &str) -> Schema {
        let format = match type_of {
            InstanceType::Integer => Some(self.name().to_lowercase()),
            _ => None,
        };
        let mut value = serde_json::json!(
            {
                "title": self.name(),
                "type": type_of,
                "description": description,
            }
        );
        if let Some(format) = format {
            value["format"] = serde_json::json!(format);
        }

        let metadata = serde_json::from_value(value).unwrap();
        Schema::Object(SchemaObject { metadata: Some(Box::new(metadata)), ..Default::default() })
    }
    pub fn schema(&self) -> Schema {
        match self {
            ScalarType::Empty => {
                self.schema_inner(InstanceType::Null, "Empty scalar type represents an empty value.")
            }
            ScalarType::Email => {
                self.schema_inner(
                    InstanceType::String,
                    "Field whose value conforms to the standard internet email address format as specified in HTML Spec: https://html.spec.whatwg.org/multipage/input.html#valid-e-mail-address.",
                )
            }
            ScalarType::PhoneNumber => {
                self.schema_inner(
                    InstanceType::String,
                    "A field whose value conforms to the standard E.164 format as specified in E.164 specification (https://en.wikipedia.org/wiki/E.164).",
                )
            }
            ScalarType::Date => {
                self.schema_inner(
                    InstanceType::String,
                    "A field whose value conforms to the standard date format as specified in RFC 3339 (https://datatracker.ietf.org/doc/html/rfc3339).",
                )
            }
            ScalarType::Url => {
                self.schema_inner(
                    InstanceType::String,
                    "A field whose value conforms to the standard URL format as specified in RFC 3986 (https://datatracker.ietf.org/doc/html/rfc3986).",
                )
            }
            ScalarType::JSON => {
                self.schema_inner(
                    InstanceType::Object,
                    "A field whose value conforms to the standard JSON format as specified in RFC 8259 (https://datatracker.ietf.org/doc/html/rfc8259).",
                )
            }
            ScalarType::Int8 => {
                self.schema_inner(
                    InstanceType::Integer,
                    "A field whose value is an 8-bit signed integer.",
                )
            }
            ScalarType::Int16 => {
                self.schema_inner(
                    InstanceType::Integer,
                    "A field whose value is a 16-bit signed integer.",
                )
            }
            ScalarType::Int32 => {
                self.schema_inner(
                    InstanceType::Integer,
                    "A field whose value is a 32-bit signed integer.",
                )
            }
            ScalarType::Int64 => {
                self.schema_inner(
                    InstanceType::Integer,
                    "A field whose value is a 64-bit signed integer.",
                )
            }
            ScalarType::Int128 => {
                self.schema_inner(
                    InstanceType::Integer,
                    "A field whose value is a 128-bit signed integer.",
                )
            }
            ScalarType::UInt8 => {
                self.schema_inner(
                    InstanceType::Integer,
                    "A field whose value is an 8-bit unsigned integer.",
                )
            }
            ScalarType::UInt16 => {
                self.schema_inner(
                    InstanceType::Integer,
                    "A field whose value is a 16-bit unsigned integer.",
                )
            }
            ScalarType::UInt32 => {
                self.schema_inner(
                    InstanceType::Integer,
                    "A field whose value is a 32-bit unsigned integer.",
                )
            }
            ScalarType::UInt64 => {
                self.schema_inner(
                    InstanceType::Integer,
                    "A field whose value is a 64-bit unsigned integer.",
                )
            }
            ScalarType::UInt128 => {
                self.schema_inner(
                    InstanceType::Integer,
                    "A field whose value is a 128-bit unsigned integer.",
                )
            }
            ScalarType::Bytes => {
                self.schema_inner(
                    InstanceType::String,
                    "A field whose value is a sequence of bytes.",
                )
            }
        }
    }
}

lazy_static! {
    pub static ref CUSTOM_SCALARS: HashMap<String, ScalarType> = {
        let scalars: Vec<ScalarType> = vec![
            ScalarType::Empty,
            ScalarType::Email,
            ScalarType::PhoneNumber,
            ScalarType::Date,
            ScalarType::Url,
            ScalarType::JSON,
            ScalarType::Int8,
            ScalarType::Int16,
            ScalarType::Int32,
            ScalarType::Int64,
            ScalarType::Int128,
            ScalarType::UInt8,
            ScalarType::UInt16,
            ScalarType::UInt32,
            ScalarType::UInt64,
            ScalarType::UInt128,
            ScalarType::Bytes,
        ];
        let mut hm = HashMap::new();

        for scalar in scalars {
            hm.insert(scalar.name(), scalar);
        }
        hm
    };
}
lazy_static! {
    static ref SCALAR_TYPES: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.extend(["String", "Int", "Float", "ID", "Boolean"]);
        set.extend(CUSTOM_SCALARS.keys().map(|k| k.as_str()));
        set
    };
}

///
/// Check if the type is a predefined scalar
pub fn is_predefined_scalar(type_name: &str) -> bool {
    SCALAR_TYPES.contains(type_name)
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
        // println!("{}", ScalarType::Empty.to_string());
        // it's easy to accidentally add a different scalar type to the schema
        // this test ensures that the scalar types are correctly defined
        for (k, v) in CUSTOM_SCALARS.iter() {
            println!("{}", k);
            assert_eq!(k.clone(), get_name(v.schema()));
        }
    }
}
