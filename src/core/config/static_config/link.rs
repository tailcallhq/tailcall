use serde::{Deserialize, Serialize};

use crate::core::config::{KeyValue, Link};
use crate::core::is_default;

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
/// The acceptable types of linked files that can be loaded on bootstrap.
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

/// Used to represent external resources, such as
/// configuration – which will be merged into the config importing it –,
/// or a .proto file – which will be later used by `@grpc` directive –.
#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Debug, Clone, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkStatic {
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
    ///
    /// Custom headers for gRPC reflection server.
    #[serde(default, skip_serializing_if = "is_default")]
    pub headers: Option<Vec<KeyValue>>,
    ///
    /// Additional metadata pertaining to the linked resource.
    #[serde(default, skip_serializing_if = "is_default")]
    pub meta: Option<serde_json::Value>,
}

impl From<Link> for LinkStatic {
    fn from(link: Link) -> Self {
        Self {
            id: link.id,
            src: link.src,
            type_of: link.type_of,
            headers: link.headers,
            meta: link.meta,
        }
    }
}
