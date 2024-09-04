use serde::{Deserialize, Serialize};
use tailcall_macros::DirectiveDefinition;

use super::super::is_default;
use super::KeyValue;
use crate::core::http::Method;

#[derive(
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Debug,
    Clone,
    schemars::JsonSchema,
    strum_macros::Display,
)]
pub enum LinkType {
    #[default]
    Config,
    Protobuf,
    Script,
    Cert,
    Key,
    Operation,
    Htpasswd,
    Jwks,
    Grpc,
}

/// The @link directive allows you to import external resources, such as
/// configuration – which will be merged into the config importing it –,
/// or a .proto file – which will be later used by `@grpc` directive –.
#[derive(
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Debug,
    Clone,
    schemars::JsonSchema,
    DirectiveDefinition,
)]
#[directive_definition(repeatable, locations = "Schema")]
#[serde(deny_unknown_fields)]
pub struct Link {
    ///
    /// The id of the link. It is used to reference the link in the schema.
    #[serde(default, skip_serializing_if = "is_default")]
    pub id: Option<String>,
    ///
    /// The source of the link. It can be a URL or a path to a file.
    /// If a path is provided, it is relative to the file that imports the link.
    #[serde(default, skip_serializing_if = "is_default")]
    pub src: String,
    ///
    /// The type of the link. It can be `Config`, or `Protobuf`.
    #[serde(default, skip_serializing_if = "is_default", rename = "type")]
    pub type_of: LinkType,

    #[serde(default, skip_serializing_if = "is_default")]
    pub method: Option<Method>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub headers: Option<Vec<KeyValue>>,
}
