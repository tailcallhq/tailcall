pub use date::*;
pub use email::*;
pub use phone::*;
pub use url::*;

mod date;
mod email;
mod phone;
mod url;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_graphql_value::ConstValue;
use lazy_static::lazy_static;
use schemars::schema::{RootSchema, Schema};
use schemars::schema_for;

lazy_static! {
    pub static ref CUSTOM_SCHEMA_FOR_SCALARS: Vec<RootSchema> = vec![
        schema_for!(Email),
        schema_for!(PhoneNumber),
        schema_for!(Date),
        schema_for!(Url),
    ];
    pub static ref CUSTOM_SCALARS: HashMap<String, Arc<dyn Scalar + Send + Sync>> = {
        let scalars: Vec<Arc<dyn Scalar + Send + Sync>> = vec![
            Arc::new(Email::default()),
            Arc::new(PhoneNumber::default()),
            Arc::new(Date::default()),
            Arc::new(Url::default()),
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
        set.extend(["String", "Int", "Float", "Boolean", "ID", "JSON"]);
        set.extend(CUSTOM_SCALARS.keys().map(|k| k.as_str()));
        set
    };
}

pub fn is_scalar(type_name: &str) -> bool {
    SCALAR_TYPES.contains(type_name)
}

pub trait Scalar {
    fn validate(&self) -> fn(&ConstValue) -> bool;
    fn scalar(&self) -> Schema;
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

    use crate::scalar::CUSTOM_SCALARS;

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
            assert_eq!(k.clone(), get_name(v.scalar()));
        }
    }
}