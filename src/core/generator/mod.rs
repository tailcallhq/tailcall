mod config;
// FIXME: once moved to CLI, rename to generator. This generator module will be different from `core::generator`.
mod config_generator_cli;
mod from_json;
mod from_proto;
mod generator;
mod graphql_type;
mod json;
mod proto;
pub mod source;

pub use config::*;
pub use config_generator_cli::ConfigConsoleGenerator;
pub use from_json::{from_json, ConfigGenerationRequest};
pub use generator::Generator;
pub use json::NameGenerator;
