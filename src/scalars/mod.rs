use std::collections::HashMap;
use std::sync::Arc;

use async_graphql_value::ConstValue;
use lazy_static::lazy_static;

pub use crate::scalars::email::Email;

mod email;

lazy_static! {
    pub static ref CUSTOM_SCALARS: HashMap<String, Arc<dyn Scalar + Send + Sync>> = {
        let mut hm: HashMap<String, Arc<dyn Scalar + Send + Sync>> = HashMap::new();
        hm.insert("Email".to_string(), Arc::new(Email::default()));
        hm
    };
}

#[derive(schemars::JsonSchema)]
pub enum CustomScalar {
    Email(Email),
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
