mod from_json;
mod from_proto;
mod generator;
mod graphql_type;
mod json;
mod proto;

pub use from_json::{FromJsonGenerator, RequestSample};
pub use generator::{Generator, Input, TransformOptions};

use crate::core::counter::{Count, Counter};

pub struct NameGenerator {
    counter: Counter<u64>,
    prefix: String,
}

impl NameGenerator {
    pub fn new(prefix: &str) -> Self {
        Self { counter: Counter::new(1), prefix: prefix.to_string() }
    }

    pub fn next(&self) -> String {
        let id = self.counter.next();
        format!("{}{}", self.prefix, id)
    }
}
