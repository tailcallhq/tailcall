use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::super::is_default;

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug, Clone, schemars::JsonSchema)]
pub enum LinkType {
    #[default]
    Config,
    Protobuf,
    Key,
    Cert,
}

impl Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LinkType::Config => "Config",
            LinkType::Protobuf => "Protobuf",
            LinkType::Key => "TlsKey",
            LinkType::Cert => "TlsCert",
        })
    }
}

/// The @link directive allows you to import external resources, such as configuration – which will be merged into the config importing it –,
/// or a .proto file – which will be later used by `@grpc` directive –.
#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug, Clone, schemars::JsonSchema)]
pub struct Link {
    ///
    /// The id of the link. It is used to reference the link in the schema.
    ///
    #[serde(default, skip_serializing_if = "is_default")]
    pub id: Option<String>,
    ///
    /// The source of the link. It can be a URL or a path to a file.
    /// If a path is provided, it is relative to the file that imports the link.
    ///
    #[serde(default, skip_serializing_if = "is_default")]
    pub src: String,
    ///
    /// The type of the link. It can be `Config`, `Protobuf`, `Key` or `Cert`.
    ///
    #[serde(default, skip_serializing_if = "is_default", rename = "type")]
    pub type_of: LinkType,
}
