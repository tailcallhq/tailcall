use serde::{Deserialize, Serialize};

use super::super::is_default;

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug, Clone, schemars::JsonSchema)]
pub enum LinkType {
    #[default]
    Config,
    Protobuf,
}

/// The @link directive allows you to import external resources, such as configuration, a .proto file, a plain text file, or a GraphQL schema.
#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug, Clone, schemars::JsonSchema)]
pub struct Link {
    ///
    /// The type of the link. It can be `Config`, `GraphQL`, `Protobuf`, or `Data`.
    ///
    #[serde(default, skip_serializing_if = "is_default", rename = "type")]
    pub type_of: LinkType,
    ///
    /// The source of the link. It can be a URL or a path to a file.
    ///
    #[serde(default, skip_serializing_if = "is_default")]
    pub src: String,
    ///
    /// The id of the link. It is used to reference the link in the schema.
    ///
    #[serde(default, skip_serializing_if = "is_default")]
    pub id: Option<String>,
}
