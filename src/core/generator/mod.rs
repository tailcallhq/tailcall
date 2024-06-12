pub mod config;
mod from_generator_config;
mod from_json;
mod from_proto;
mod generator;
mod graphql_type;
mod json;
mod proto;
pub mod source;

pub use from_generator_config::Reader;
pub use from_json::{from_json, ConfigGenerationRequest};
pub use generator::Generator;
pub use json::NameGenerator;
