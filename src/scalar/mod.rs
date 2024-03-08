pub use date::*;
pub use email::*;
pub use phone::*;
pub use url::*;

mod date;
mod email;
mod phone;
mod url;

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::sync::Arc;

use async_graphql_value::ConstValue;
use lazy_static::lazy_static;
use schemars::schema::Schema;
use schemars::schema_for;

pub struct ScalarMetadata {
    pub instance: Arc<dyn Scalar + Send + Sync>,
    pub schema: Schema,
}

lazy_static! {
    pub static ref CUSTOM_SCALARS: HashMap<String, ScalarMetadata> = {
        let scalars = vec![
            ScalarMetadata {
                instance: Arc::new(Email::default()),
                schema: Schema::Object(schema_for!(Email).schema),
            },
            ScalarMetadata {
                instance: Arc::new(PhoneNumber::default()),
                schema: Schema::Object(schema_for!(PhoneNumber).schema),
            },
            ScalarMetadata {
                instance: Arc::new(Date::default()),
                schema: Schema::Object(schema_for!(Date).schema),
            },
            ScalarMetadata {
                instance: Arc::new(Url::default()),
                schema: Schema::Object(schema_for!(Url).schema),
            },
        ];
        let mut hm = HashMap::new();

        for scalar in scalars {
            hm.insert(scalar.instance.to_string(), scalar);
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

pub trait Scalar: Display {
    fn validate(&self) -> fn(&ConstValue) -> bool;
}

pub fn get_scalar(name: &str) -> fn(&ConstValue) -> bool {
    CUSTOM_SCALARS
        .get(name)
        .map(|v| v.instance.validate())
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
            assert_eq!(k.clone(), get_name(v.schema.clone()));
        }
    }
}
