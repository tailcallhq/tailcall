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
use std::sync::Arc;

use async_graphql_value::ConstValue;
use lazy_static::lazy_static;
use schemars::schema::Schema;

lazy_static! {
    pub static ref CUSTOM_SCALARS: HashMap<String, Arc<dyn Scalar + Send + Sync>> = {
        let scalars: Vec<Arc<dyn Scalar + Send + Sync>> = vec![
            Arc::new(Email::default()),
            Arc::new(PhoneNumber::default()),
            Arc::new(Date::default()),
            Arc::new(Url::default()),
            Arc::new(JSON::default()),
            Arc::new(Empty::default()),
            Arc::new(Int8::default()),
            Arc::new(Int16::default()),
            Arc::new(Int32::default()),
            Arc::new(Int64::default()),
            Arc::new(Int128::default()),
            Arc::new(UInt8::default()),
            Arc::new(UInt16::default()),
            Arc::new(UInt32::default()),
            Arc::new(UInt64::default()),
            Arc::new(UInt128::default()),
            Arc::new(Bytes::default()),
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

pub trait Scalar {
    fn validate(&self) -> fn(&ConstValue) -> bool;
    fn schema(&self) -> Schema;
    fn name(&self) -> String {
        std::any::type_name::<Self>()
            .split("::")
            .last()
            .unwrap()
            .to_string()
    }
}

pub fn get_scalar(name: &str) -> fn(&ConstValue) -> bool {
    CUSTOM_SCALARS
        .get(name)
        .map(|v| v.validate())
        .unwrap_or(|_| true)
}

#[cfg(test)]
mod test {
    use schemars::schema::Schema;

    use crate::core::scalar::CUSTOM_SCALARS;

    /// generates test asserts for valid scalar inputs
    #[macro_export]
    macro_rules! test_scalar_valid {
        ($ty: ty, $($value: expr),+) => {
            #[test]
            fn test_scalar_valid() {
                let value = <$ty>::default();

                $(
                    assert!(value.validate()(&$value));
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
                let value = <$ty>::default();

                $(
                    assert!(!value.validate()(&$value));
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
