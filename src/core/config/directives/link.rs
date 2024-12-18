use serde::{Deserialize, Serialize};
use tailcall_macros::DirectiveDefinition;

use crate::core::config::KeyValue;
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
pub enum LinkType {
    #[default]
    /// Points to another Tailcall Configuration file. The imported
    /// configuration will be merged into the importing configuration.
    Config,

    /// Points to a Protobuf file. The imported Protobuf file will be used by
    /// the `@grpc` directive. If your API exposes a reflection endpoint, you
    /// should set the type to `Grpc` instead.
    Protobuf,

    /// Points to a JS file. The imported JS file will be used by the `@js`
    /// directive.
    Script,

    /// Points to a Cert file. The imported Cert file will be used by the server
    /// to serve over HTTPS.
    Cert,

    /// Points to a Key file. The imported Key file will be used by the server
    /// to serve over HTTPS.
    Key,

    /// A trusted document that contains GraphQL operations (queries, mutations)
    /// that can be exposed a REST API using the `@rest` directive.
    Operation,

    /// Points to a Htpasswd file. The imported Htpasswd file will be used by
    /// the server to authenticate users.
    Htpasswd,

    /// Points to a Jwks file. The imported Jwks file will be used by the server
    /// to authenticate users.
    Jwks,

    /// Points to a reflection endpoint. The imported reflection endpoint will
    /// be used by the `@grpc` directive to resolve data from gRPC services.
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
    ///
    /// Custom headers for gRPC reflection server.
    #[serde(default, skip_serializing_if = "is_default")]
    pub headers: Option<Vec<KeyValue>>,
    ///
    /// Additional metadata pertaining to the linked resource.
    #[serde(default, skip_serializing_if = "is_default")]
    pub meta: Option<serde_json::Value>,
    ///
    /// The proto paths to be used when resolving dependencies.
    /// Only valid when [`Link::type_of`] is [`LinkType::Protobuf`]
    #[serde(default, skip_serializing_if = "is_default")]
    pub proto_paths: Option<Vec<String>>,
}
