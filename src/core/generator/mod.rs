mod from_json;
mod from_proto;
mod generator;
mod graphql_type;
mod json;
mod proto;
pub mod source;

pub use from_json::{FromJsonGenerator, RequestSample};
pub use generator::{ConfigInput, Generator, JsonInput, ProtoInput};
pub use json::NameGenerator;

use super::config::Config;

pub trait Generate {
    fn generate(&self) -> anyhow::Result<Config>;
}
