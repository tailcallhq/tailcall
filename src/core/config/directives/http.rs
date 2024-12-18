use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall_macros::{DirectiveDefinition, InputDefinition};

use crate::core::config::{Encoding, KeyValue, URLQuery};
use crate::core::http::Method;
use crate::core::is_default;
use crate::core::json::JsonSchema;

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    schemars::JsonSchema,
    DirectiveDefinition,
    InputDefinition,
)]
#[directive_definition(repeatable, locations = "FieldDefinition, Object")]
#[serde(deny_unknown_fields)]
/// The @http operator indicates that a field or node is backed by a REST API.
///
/// For instance, if you add the @http operator to the `users` field of the
/// Query type with a path argument of `"/users"`, it signifies that the `users`
/// field is backed by a REST API. The path argument specifies the path of the
/// REST API. In this scenario, the GraphQL server will make a GET request to
/// the API endpoint specified when the `users` field is queried.
pub struct Http {
    #[serde(rename = "onRequest", default, skip_serializing_if = "is_default")]
    /// onRequest field in @http directive gives the ability to specify the
    /// request interception handler.
    pub on_request: Option<String>,

    /// This refers to URL of the API.
    pub url: String,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The body of the API call. It's used for methods like POST or PUT that
    /// send data to the server. You can pass it as a static object or use a
    /// Mustache template with object to substitute variables from the GraphQL
    /// variables.
    pub body: Option<Value>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The `encoding` parameter specifies the encoding of the request body. It
    /// can be `ApplicationJson` or `ApplicationXWwwFormUrlEncoded`. @default
    /// `ApplicationJson`.
    pub encoding: Encoding,

    #[serde(rename = "batchKey", default, skip_serializing_if = "is_default")]
    /// The `batchKey` dictates the path Tailcall will follow to group the returned items from the batch request. For more details please refer out [n + 1 guide](https://tailcall.run/docs/guides/n+1#solving-using-batching).
    pub batch_key: Vec<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The `headers` parameter allows you to customize the headers of the HTTP
    /// request made by the `@http` operator. It is used by specifying a
    /// key-value map of header names and their values.
    pub headers: Vec<KeyValue>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Schema of the input of the API call. It is automatically inferred in
    /// most cases.
    pub input: Option<JsonSchema>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// This refers to the HTTP method of the API call. Commonly used methods
    /// include `GET`, `POST`, `PUT`, `DELETE` etc. @default `GET`.
    pub method: Method,

    #[serde(default, skip_serializing_if = "is_default")]
    /// Schema of the output of the API call. It is automatically inferred in
    /// most cases.
    pub output: Option<JsonSchema>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// This represents the query parameters of your API call. You can pass it
    /// as a static object or use Mustache template for dynamic parameters.
    /// These parameters will be added to the URL.
    /// NOTE: Query parameter order is critical for batching in Tailcall. The
    /// first parameter referencing a field in the current value using mustache
    /// syntax is automatically selected as the batching parameter.
    pub query: Vec<URLQuery>,
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
