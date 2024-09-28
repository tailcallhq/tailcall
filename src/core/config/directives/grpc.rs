use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall_macros::{DirectiveDefinition, InputDefinition};

use crate::core::config::{Batch, KeyValue};
use crate::core::is_default;

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    InputDefinition,
    DirectiveDefinition,
)]
#[directive_definition(locations = "FieldDefinition")]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
/// The @grpc operator indicates that a field or node is backed by a gRPC API.
///
/// For instance, if you add the @grpc operator to the `users` field of the
/// Query type with a service argument of `NewsService` and method argument of
/// `GetAllNews`, it signifies that the `users` field is backed by a gRPC API.
/// The `service` argument specifies the name of the gRPC service.
/// The `method` argument specifies the name of the gRPC method.
/// In this scenario, the GraphQL server will make a gRPC request to the gRPC
/// endpoint specified when the `users` field is queried.
pub struct Grpc {
    #[serde(rename = "baseURL", default, skip_serializing_if = "is_default")]
    /// This refers to the base URL of the API. If not specified, the default
    /// base URL is the one specified in the `@upstream` operator.
    pub base_url: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    /// This refers to the arguments of your gRPC call. You can pass it as a
    /// static object or use Mustache template for dynamic parameters. These
    /// parameters will be added in the body in `protobuf` format.
    pub body: Option<Value>,
    #[serde(rename = "batchKey", default, skip_serializing_if = "is_default")]
    /// The `batchKey` dictates the path Tailcall will follow to group the returned items from the batch request. For more details please refer out [n + 1 guide](https://tailcall.run/docs/guides/n+1#solving-using-batching).
    pub batch_key: Vec<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    /// The `headers` parameter allows you to customize the headers of the HTTP
    /// request made by the `@grpc` operator. It is used by specifying a
    /// key-value map of header names and their values. Note: content-type is
    /// automatically set to application/grpc
    pub headers: Vec<KeyValue>,
    /// This refers to the gRPC method you're going to call. For instance
    /// `GetAllNews`.
    pub method: String,
    #[serde(default, skip_serializing_if = "is_default")]
    pub allowed_headers: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub batch: Option<Batch>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub on_request: Option<String>,
}
