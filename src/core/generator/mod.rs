mod from_json;
mod from_proto;
mod generator;
mod graphql_type;
mod json;
mod proto;

pub use from_json::{FromJsonGenerator, RequestSample};
pub use generator::{Generator, Input};

use super::counter::AtomicCounter;
use crate::core::counter::Count;

pub struct NameGenerator {
    counter: AtomicCounter<u64>,
    prefix: String,
}

impl NameGenerator {
    pub fn new(prefix: &str) -> Self {
        Self { counter: AtomicCounter::new(1), prefix: prefix.to_string() }
    }

    pub fn next(&self) -> String {
        let id = self.counter.next();
        format!("{}{}", self.prefix, id)
    }
}
