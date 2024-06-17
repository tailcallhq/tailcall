mod config;
mod from_json;
mod from_proto;
mod generator;
mod graphql_type;
mod json;
mod proto;
pub mod source;

pub use config::*;
pub use from_json::{from_json, FromJson, RequestSample};
pub use generator::{ConfigInput, Generator, JsonInput, ProtoInput};
pub use json::NameGenerator;

use super::config::Config;
use super::valid::Valid;

pub trait Generate {
    type Error;
    fn generate(&self) -> Valid<Config, Self::Error>;
}
