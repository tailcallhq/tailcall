mod from_json;
pub mod config;
mod from_proto;
mod generator;
mod graphql_type;
mod json;
mod proto;
mod from_generalized_config;
pub mod source;

pub use from_json::{from_json, ConfigGenerationRequest};
pub use json::NameGenerator;
pub use generator::Generator;
pub use from_generalized_config::FromGeneralizedConfig;
