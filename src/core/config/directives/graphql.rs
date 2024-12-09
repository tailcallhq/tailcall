use serde::{Deserialize, Serialize};
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
    DirectiveDefinition,
    InputDefinition,
)]
#[directive_definition(repeatable, locations = "FieldDefinition, Object")]
#[serde(deny_unknown_fields)]
/// The @graphQL operator allows to specify GraphQL API server request to fetch
/// data from.
pub struct GraphQL {
    #[serde(default, skip_serializing_if = "is_default")]
    /// Named arguments for the requested field. More info [here](https://tailcall.run/docs/guides/operators/#args)
    pub args: Option<Vec<KeyValue>>,

    /// This refers URL of the API.
    pub url: String,

    #[serde(default, skip_serializing_if = "is_default")]
    /// If the upstream GraphQL server supports request batching, you can
    /// specify the 'batch' argument to batch several requests into a single
    /// batch request.
    ///
    /// Make sure you have also specified batch settings to the `@upstream` and
    /// to the `@graphQL` operator.
    pub batch: bool,

    #[serde(default, skip_serializing_if = "is_default")]
    /// The headers parameter allows you to customize the headers of the GraphQL
    /// request made by the `@graphQL` operator. It is used by specifying a
    /// key-value map of header names and their values.
    pub headers: Vec<KeyValue>,

    /// Specifies the root field on the upstream to request data from. This maps
    /// a field in your schema to a field in the upstream schema. When a query
    /// is received for this field, Tailcall requests data from the
    /// corresponding upstream field.
    pub name: String,
    #[serde(default, skip_serializing_if = "is_default")]
    /// Enables deduplication of IO operations to enhance performance.
    ///
    /// This flag prevents duplicate IO requests from being executed
    /// concurrently, reducing resource load. Caution: May lead to issues
    /// with APIs that expect unique results for identical inputs, such as
    /// nonce-based APIs.
    pub dedupe: bool,
}
