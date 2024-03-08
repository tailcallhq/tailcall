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
use schemars::schema::Schema;
use schemars::schema_for;

pub struct ScalarMetadata {
    pub instance: Arc<dyn Scalar + Send + Sync>,
    pub schema: Schema,
}

lazy_static! {
    pub static ref CUSTOM_SCALARS: HashMap<String, ScalarMetadata> = {
        let mut hm = HashMap::new();
        hm.insert(
            "Email".to_string(),
            ScalarMetadata {
                instance: Arc::new(Email::default()),
                schema: Schema::Object(schema_for!(Email).schema),
            },
        );
        hm.insert(
            "PhoneNumber".to_string(),
            ScalarMetadata {
                instance: Arc::new(PhoneNumber::default()),
                schema: Schema::Object(schema_for!(PhoneNumber).schema),
            },
        );
        hm.insert(
            "Date".to_string(),
            ScalarMetadata {
                instance: Arc::new(Date::default()),
                schema: Schema::Object(schema_for!(Date).schema),
            },
        );
        hm.insert(
            "Url".to_string(),
            ScalarMetadata {
                instance: Arc::new(Url::default()),
                schema: Schema::Object(schema_for!(Url).schema),
            },
        );
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
}

pub fn get_scalar(name: &str) -> fn(&ConstValue) -> bool {
    CUSTOM_SCALARS
        .get(name)
        .map(|v| v.instance.validate())
        .unwrap_or(|_| true)
}
