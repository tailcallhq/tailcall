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
#[directive_definition(locations = "FieldDefinition, Object")]
#[serde(deny_unknown_fields)]
/// The @http operator indicates that a field or node is backed by a REST API.
pub struct Http {
    #[serde(rename = "onRequest", default, skip_serializing_if = "is_default")]
    pub on_request: Option<String>,

    pub url: String,

    #[serde(default, skip_serializing_if = "is_default")]
    pub body: Option<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub encoding: Encoding,

    #[serde(rename = "batchKey", default, skip_serializing_if = "is_default")]
    pub batch_key: Vec<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub headers: Vec<KeyValue>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub input: Option<JsonSchema>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub method: Method,

    #[serde(default, skip_serializing_if = "is_default")]
    pub output: Option<JsonSchema>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub query: Vec<URLQuery>,

    #[serde(default, skip_serializing_if = "is_default")]
    pub dedupe: Option<bool>,

    pub select: Option<Value>,
}

impl Http {
    /// Validates that query parameters don't contain objects
    pub fn validate_query_params(&self) -> Result<(), String> {
        for query in &self.query {
            if query.value.contains("{{.args.") {
                let arg_path = query.value
                    .trim_start_matches("{{.args.")
                    .trim_end_matches("}}");
                
                if arg_path.contains('.') {
                    return Err(format!(
                        "Invalid query parameter type for '{}'. Expected a Scalar but received an Object.",
                        query.key
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_query_params_with_scalar() {
        let http = Http {
            url: "test".to_string(),
            query: vec![URLQuery {
                key: "id".to_string(),
                value: "{{.args.id}}".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(http.validate_query_params().is_ok());
    }

    #[test]
    fn test_validate_query_params_with_object() {
        let http = Http {
            url: "test".to_string(),
            query: vec![URLQuery {
                key: "nested".to_string(),
                value: "{{.args.criteria.maritalStatus}}".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(http.validate_query_params().is_err());
    }
}