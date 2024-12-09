use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall_macros::{DirectiveDefinition, InputDefinition};

use crate::core::config::KeyValue;
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
#[directive_definition(repeatable, locations = "FieldDefinition, Object")]
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
    /// This refers to URL of the API.
    pub url: String,
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
    /// Enables deduplication of IO operations to enhance performance.
    ///
    /// This flag prevents duplicate IO requests from being executed
    /// concurrently, reducing resource load. Caution: May lead to issues
    /// with APIs that expect unique results for identical inputs, such as
    /// nonce-based APIs.
    pub dedupe: Option<bool>,

    /// You can use `select` with mustache syntax to re-construct the directives
    /// response to the desired format. This is useful when data are deeply
    /// nested or want to keep specific fields only from the response.
    ///
    /// * EXAMPLE 1: if we have a call that returns `{ "user": { "items": [...],
    ///   ... } ... }` we can use `"{{.user.items}}"`, to extract the `items`.
    /// * EXAMPLE 2: if we have a call that returns `{ "foo": "bar", "fizz": {
    ///   "buzz": "eggs", ... }, ... }` we can use { foo: "{{.foo}}", buzz:
    ///   "{{.fizz.buzz}}" }`
    pub select: Option<Value>,

    /// Specifies a JavaScript function to be executed after receiving the
    /// response body. This function can modify or transform the response
    /// body before it's sent back to the client.
    #[serde(rename = "onResponseBody", default, skip_serializing_if = "is_default")]
    pub on_response_body: Option<String>,
}
