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

lazy_static! {
    static ref CUSTOM_SCALARS: HashMap<String, Arc<dyn Scalar + Send + Sync>> = {
        let mut hm: HashMap<String, Arc<dyn Scalar + Send + Sync>> = HashMap::new();
        hm.insert("Email".to_string(), Arc::new(Email::default()));
        hm.insert("PhoneNumber".to_string(), Arc::new(PhoneNumber::default()));
        hm.insert("Date".to_string(), Arc::new(Date::default()));
        hm.insert("Url".to_string(), Arc::new(Url::default()));
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

#[derive(schemars::JsonSchema)]
pub enum CustomScalar {
    Email(Email),
    PhoneNumber(PhoneNumber),
    Date(Date),
    Url(Url),
}

pub trait Scalar {
    fn validate(&self) -> fn(&ConstValue) -> bool;
}

pub fn get_scalar(name: &str) -> fn(&ConstValue) -> bool {
    CUSTOM_SCALARS
        .get(name)
        .map(|v| v.validate())
        .unwrap_or(|_| true)
}
