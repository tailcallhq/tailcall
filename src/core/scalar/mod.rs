pub use bytes::*;
pub use date::*;
pub use email::*;
pub use empty::*;
pub use int128::*;
pub use int16::*;
pub use int32::*;
pub use int64::*;
pub use int8::*;
pub use json::*;
pub use phone::*;
pub use uint128::*;
pub use uint16::*;
pub use uint32::*;
pub use uint64::*;
pub use uint8::*;
pub use url::*;

mod bytes;
mod date;
mod email;
mod empty;
mod int128;
mod int16;
mod int32;
mod int64;
mod int8;
mod json;
mod phone;
mod uint128;
mod uint16;
mod uint32;
mod uint64;
mod uint8;
mod url;

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use schemars::schema::Schema;
use schemars::schema_for;

use crate::core::json::JsonLike;

#[derive(schemars::JsonSchema, Debug, Clone)]
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

impl ScalarType {
    pub fn validate<'a, Value: JsonLike<'a>>(&self) -> fn(&'a Value) -> bool {
        match self {
            ScalarType::Empty => Empty::default().validate(),
            ScalarType::Email => Email::default().validate(),
            ScalarType::PhoneNumber => PhoneNumber::default().validate(),
            ScalarType::Date => Date::default().validate(),
            ScalarType::Url => Url::default().validate(),
            ScalarType::JSON => JSON::default().validate(),
            ScalarType::Int8 => Int8::default().validate(),
            ScalarType::Int16 => Int16::default().validate(),
            ScalarType::Int32 => Int32::default().validate(),
            ScalarType::Int64 => Int64::default().validate(),
            ScalarType::Int128 => Int128::default().validate(),
            ScalarType::UInt8 => UInt8::default().validate(),
            ScalarType::UInt16 => UInt16::default().validate(),
            ScalarType::UInt32 => UInt32::default().validate(),
            ScalarType::UInt64 => UInt64::default().validate(),
            ScalarType::UInt128 => UInt128::default().validate(),
            ScalarType::Bytes => Bytes::default().validate(),
        }
    }

    pub fn schema(&self) -> Schema {
        match self {
            ScalarType::Empty => Empty::default().schema(),
            ScalarType::Email => Email::default().schema(),
            ScalarType::PhoneNumber => PhoneNumber::default().schema(),
            ScalarType::Date => Date::default().schema(),
            ScalarType::Url => Url::default().schema(),
            ScalarType::JSON => JSON::default().schema(),
            ScalarType::Int8 => Int8::default().schema(),
            ScalarType::Int16 => Int16::default().schema(),
            ScalarType::Int32 => Int32::default().schema(),
            ScalarType::Int64 => Int64::default().schema(),
            ScalarType::Int128 => Int128::default().schema(),
            ScalarType::UInt8 => UInt8::default().schema(),
            ScalarType::UInt16 => UInt16::default().schema(),
            ScalarType::UInt32 => UInt32::default().schema(),
            ScalarType::UInt64 => UInt64::default().schema(),
            ScalarType::UInt128 => UInt128::default().schema(),
            ScalarType::Bytes => Bytes::default().schema(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            ScalarType::Empty => "Empty".to_string(),
            ScalarType::Email => "Email".to_string(),
            ScalarType::PhoneNumber => "PhoneNumber".to_string(),
            ScalarType::Date => "Date".to_string(),
            ScalarType::Url => "Url".to_string(),
            ScalarType::JSON => "JSON".to_string(),
            ScalarType::Int8 => "Int8".to_string(),
            ScalarType::Int16 => "Int16".to_string(),
            ScalarType::Int32 => "Int32".to_string(),
            ScalarType::Int64 => "Int64".to_string(),
            ScalarType::Int128 => "Int128".to_string(),
            ScalarType::UInt8 => "UInt8".to_string(),
            ScalarType::UInt16 => "UInt16".to_string(),
            ScalarType::UInt32 => "UInt32".to_string(),
            ScalarType::UInt64 => "UInt64".to_string(),
            ScalarType::UInt128 => "UInt128".to_string(),
            ScalarType::Bytes => "Bytes".to_string(),
        }
    }
}

pub fn get_scalar(name: &str) -> ScalarType {
    CUSTOM_SCALARS
        .get(name)
        .cloned()
        .unwrap_or(ScalarType::Empty)
}

lazy_static! {
    pub static ref CUSTOM_SCALARS: HashMap<String, ScalarType> = {
        let scalars: Vec<ScalarType> = vec![
            ScalarType::Email,
            ScalarType::PhoneNumber,
            ScalarType::Date,
            ScalarType::Url,
            ScalarType::JSON,
            ScalarType::Empty,
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

    use crate::core::scalar::{ScalarType, CUSTOM_SCALARS};

    /// generates test asserts for valid scalar inputs
    #[macro_export]
    macro_rules! test_scalar_valid {
        ($ty: ty, $($value: expr),+) => {
            #[test]
            fn test_scalar_valid() {
                let value = ScalarType::$ty;

                $(
                    assert!(value.validate::<async_graphql_value::ConstValue>()(&$value));
                )+
            }
        };
    }

    // generates test asserts for invalid scalar inputs
    #[macro_export]
    macro_rules! test_scalar_invalid {
        ($ty: ty, $($value: expr),+) => {
            #[test]
            fn test_scalar_invalid() {
                let value = ScalarType::$ty;

                $(
                    assert!(!value.validate::<async_graphql_value::ConstValue>()(&$value));
                )+
            }
        };
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
