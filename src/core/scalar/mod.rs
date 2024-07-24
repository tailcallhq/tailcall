use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use lazy_static::lazy_static;
use schemars::schema::Schema;

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

impl ScalarType {
    pub fn validate<'a, Value: JsonLike<'a> + 'a>(&self, value: &'a Value) -> bool {
        match self {
            ScalarType::JSON => true,
            ScalarType::Empty => true,
            ScalarType::Email => value.as_str().map_or(false, |s| {
                async_graphql::validators::email(&s.to_string()).is_ok()
            }),
            ScalarType::PhoneNumber => value
                .as_str()
                .map_or(false, |s| phonenumber::parse(None, s).is_ok()),
            ScalarType::Date => value
                .as_str()
                .map_or(false, |s| chrono::DateTime::parse_from_rfc3339(s).is_ok()),
            ScalarType::Url => value.as_str().map_or(false, |s| url::Url::parse(s).is_ok()),
            ScalarType::Bytes => value.as_str().is_some(),

            ScalarType::Int8 => value.as_i64().map_or(false, |n| i8::try_from(n).is_ok()),
            ScalarType::Int16 => value.as_i64().map_or(false, |n| i16::try_from(n).is_ok()),
            ScalarType::Int32 => value.as_i64().map_or(false, |n| i32::try_from(n).is_ok()),
            ScalarType::Int64 => value.as_str().map_or(false, |s| s.parse::<i64>().is_ok()),

            ScalarType::UInt8 => value.as_u64().map_or(false, |n| u8::try_from(n).is_ok()),
            ScalarType::UInt16 => value.as_u64().map_or(false, |n| u16::try_from(n).is_ok()),
            ScalarType::UInt32 => value.as_u64().map_or(false, |n| u32::try_from(n).is_ok()),

            ScalarType::UInt64 => value.as_str().map_or(false, |s| s.parse::<u64>().is_ok()),
            ScalarType::Int128 => value.as_str().map_or(false, |s| s.parse::<i128>().is_ok()),
            ScalarType::UInt128 => value.as_str().map_or(false, |s| s.parse::<u128>().is_ok()),
        }
    }
    pub fn get_scalar(name: &str) -> ScalarType {
        CUSTOM_SCALARS.get(name).cloned().unwrap_or(Self::Empty)
    }
    pub fn name(&self) -> String {
        self.to_string().to_lowercase()
    }
    pub fn scalar_definition(&self) -> async_graphql::parser::types::TypeSystemDefinition {
        let schemars = self.schema();
        tailcall_typedefs_common::scalar_definition::into_scalar_definition(schemars, &self.name())
    }
    pub fn schema(&self) -> Schema {
        let schemars = schemars::schema::RootSchema::default();
        Schema::Object(schemars.schema)
    }
}

lazy_static! {
    // TODO: rename
    pub static ref CUSTOM_SCALARS: HashMap<String, ScalarType> = {
        let scalars: Vec<ScalarType> = vec![
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
        // it's easy to accidentally add a different scalar type to the schema
        // this test ensures that the scalar types are correctly defined
        for (k, v) in CUSTOM_SCALARS.iter() {
            assert_eq!(k.clone(), get_name(v.schema()));
        }
    }
}
