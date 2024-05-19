use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::merge_key_value_vecs;
use crate::core::config::headers::Headers;
use crate::core::config::KeyValue;
use crate::core::is_default;
use crate::core::macros::MergeRight;
use crate::core::merge_right::MergeRight;

#[derive(
    Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema, MergeRight,
)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
/// The `@server` directive, when applied at the schema level, offers a
/// comprehensive set of server configurations. It dictates how the server
/// behaves and helps tune tailcall for various use-cases.
pub struct Server {
    #[serde(default, skip_serializing_if = "is_default")]
    /// `apolloTracing` exposes GraphQL query performance data, including
    /// execution time of queries and individual resolvers.
    pub apollo_tracing: Option<bool>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `batchRequests` combines multiple requests into one, improving
    /// performance but potentially introducing latency and complicating
    /// debugging. Use judiciously. @default `false`.
    pub batch_requests: Option<bool>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `headers` contains key-value pairs that are included as default headers
    /// in server responses, allowing for consistent header management across
    /// all responses.
    pub headers: Option<Headers>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `globalResponseTimeout` sets the maximum query duration before
    /// termination, acting as a safeguard against long-running queries.
    pub global_response_timeout: Option<i64>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `hostname` sets the server hostname.
    pub hostname: Option<String>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `introspection` allows clients to fetch schema information directly,
    /// aiding tools and applications in understanding available types, fields,
    /// and operations. @default `true`.
    pub introspection: Option<bool>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `pipelineFlush` allows to control flushing behavior of the server
    /// pipeline.
    pub pipeline_flush: Option<bool>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `port` sets the Tailcall running port. @default `8000`.
    pub port: Option<u16>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `queryValidation` checks incoming GraphQL queries against the schema,
    /// preventing errors from invalid queries. Can be disabled for performance.
    /// @default `false`.
    pub query_validation: Option<bool>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `responseValidation` Tailcall automatically validates responses from
    /// upstream services using inferred schema. @default `false`.
    pub response_validation: Option<bool>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// A link to an external JS file that listens on every HTTP request
    /// response event.
    pub script: Option<ScriptOptions>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `showcase` enables the /showcase/graphql endpoint.
    pub showcase: Option<bool>,

    #[serde(default, skip_serializing_if = "is_default")]
    #[merge_right(merge_right_fn = "merge_right_vars")]
    /// This configuration defines local variables for server operations. Useful
    /// for storing constant configurations, secrets, or shared information.
    pub vars: Vec<KeyValue>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `version` sets the HTTP version for the server. Options are `HTTP1` and
    /// `HTTP2`. @default `HTTP1`.
    pub version: Option<HttpVersion>,

    #[serde(default, skip_serializing_if = "is_default")]
    /// `workers` sets the number of worker threads. @default the number of
    /// system cores.
    pub workers: Option<usize>,
}

fn merge_right_vars(mut left: Vec<KeyValue>, right: Vec<KeyValue>) -> Vec<KeyValue> {
    left = merge_key_value_vecs(&left, &right);
    left
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, schemars::JsonSchema, MergeRight)]
#[serde(rename_all = "camelCase")]
pub struct ScriptOptions {
    pub timeout: Option<u64>,
}

#[derive(
    Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Default, schemars::JsonSchema, MergeRight,
)]
pub enum HttpVersion {
    #[default]
    HTTP1,
    HTTP2,
}

impl Server {
    pub fn enable_apollo_tracing(&self) -> bool {
        self.apollo_tracing.unwrap_or(false)
    }

    pub fn get_global_response_timeout(&self) -> i64 {
        self.global_response_timeout.unwrap_or(0)
    }

    pub fn get_workers(&self) -> usize {
        self.workers.unwrap_or(num_cpus::get())
    }

    pub fn get_port(&self) -> u16 {
        self.port.unwrap_or(8000)
    }
    pub fn enable_http_validation(&self) -> bool {
        self.response_validation.unwrap_or(false)
    }
    pub fn enable_cache_control(&self) -> bool {
        self.headers
            .as_ref()
            .map(|h| h.enable_cache_control())
            .unwrap_or(false)
    }
    pub fn enable_set_cookies(&self) -> bool {
        self.headers
            .as_ref()
            .map(|h| h.set_cookies())
            .unwrap_or(false)
    }
    pub fn enable_introspection(&self) -> bool {
        self.introspection.unwrap_or(true)
    }
    pub fn enable_query_validation(&self) -> bool {
        self.query_validation.unwrap_or(false)
    }
    pub fn enable_batch_requests(&self) -> bool {
        self.batch_requests.unwrap_or(false)
    }
    pub fn enable_showcase(&self) -> bool {
        self.showcase.unwrap_or(false)
    }

    pub fn get_hostname(&self) -> String {
        self.hostname.clone().unwrap_or("127.0.0.1".to_string())
    }

    pub fn get_vars(&self) -> BTreeMap<String, String> {
        self.vars
            .clone()
            .iter()
            .map(|kv| (kv.key.clone(), kv.value.clone()))
            .collect()
    }

    pub fn get_response_headers(&self) -> Vec<(String, String)> {
        self.headers
            .as_ref()
            .map(|h| h.custom.clone())
            .map_or(Vec::new(), |headers| {
                headers
                    .iter()
                    .map(|kv| (kv.key.clone(), kv.value.clone()))
                    .collect()
            })
    }

    pub fn get_experimental_headers(&self) -> BTreeSet<String> {
        self.headers
            .as_ref()
            .map(|h| h.experimental.clone().unwrap_or_default())
            .unwrap_or_default()
    }

    pub fn get_version(self) -> HttpVersion {
        self.version.unwrap_or(HttpVersion::HTTP1)
    }

    pub fn get_pipeline_flush(&self) -> bool {
        self.pipeline_flush.unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::ScriptOptions;

    fn server_with_script_options(so: ScriptOptions) -> Server {
        Server { script: Some(so), ..Default::default() }
    }

    #[test]
    fn script_options_merge_both() {
        let a = server_with_script_options(ScriptOptions { timeout: Some(100) });
        let b = server_with_script_options(ScriptOptions { timeout: Some(200) });
        let merged = a.merge_right(b);
        let expected = ScriptOptions { timeout: Some(200) };
        assert_eq!(merged.script, Some(expected));
    }

    #[test]
    fn script_options_merge_first() {
        let a = server_with_script_options(ScriptOptions { timeout: Some(100) });
        let b = server_with_script_options(ScriptOptions { timeout: None });
        let merged = a.merge_right(b);
        let expected = ScriptOptions { timeout: Some(100) };
        assert_eq!(merged.script, Some(expected));
    }

    #[test]
    fn script_options_merge_second() {
        let a = server_with_script_options(ScriptOptions { timeout: None });
        let b = server_with_script_options(ScriptOptions { timeout: Some(100) });
        let merged = a.merge_right(b);
        let expected = ScriptOptions { timeout: Some(100) };
        assert_eq!(merged.script, Some(expected));
    }

    #[test]
    fn script_options_merge_second_default() {
        let a = server_with_script_options(ScriptOptions { timeout: Some(100) });
        let b = Server::default();
        let merged = a.merge_right(b);
        let expected = ScriptOptions { timeout: Some(100) };
        assert_eq!(merged.script, Some(expected));
    }

    #[test]
    fn script_options_merge_first_default() {
        let a = Server::default();
        let b = server_with_script_options(ScriptOptions { timeout: Some(100) });
        let merged = a.merge_right(b);
        let expected = ScriptOptions { timeout: Some(100) };
        assert_eq!(merged.script, Some(expected));
    }

    fn get_default_left_vec() -> Vec<KeyValue> {
        [
            KeyValue { key: "left".to_string(), value: "From Left".to_string() },
            KeyValue { key: "1".to_string(), value: "1, Left".to_string() },
            KeyValue { key: "2".to_string(), value: "2, Left".to_string() },
        ]
        .to_vec()
    }

    fn get_default_right_vec() -> Vec<KeyValue> {
        [
            KeyValue { key: "right".to_string(), value: "From Right".to_string() },
            KeyValue { key: "1".to_string(), value: "1, Right".to_string() },
            KeyValue { key: "2".to_string(), value: "2, Right".to_string() },
        ]
        .to_vec()
    }

    fn get_sorted_expected_merge_value() -> Vec<KeyValue> {
        let mut res = [
            KeyValue { key: "right".to_string(), value: "From Right".to_string() },
            KeyValue { key: "left".to_string(), value: "From Left".to_string() },
            KeyValue { key: "1".to_string(), value: "1, Right".to_string() },
            KeyValue { key: "2".to_string(), value: "2, Right".to_string() },
        ]
        .to_vec();
        res.sort_by(|a, b| a.key.cmp(&b.key));
        res
    }

    #[test]
    fn check_merge_vec_fn() {
        let left_vec = get_default_left_vec();
        let right_vec = get_default_right_vec();
        let expected_vec = get_sorted_expected_merge_value();

        let mut merge_vec = merge_key_value_vecs(&left_vec, &right_vec);
        merge_vec.sort_by(|a, b| a.key.cmp(&b.key));

        assert_eq!(merge_vec, expected_vec)
    }

    #[test]
    fn check_merge_right_fn() {
        let left_vec = get_default_left_vec();
        let right_vec = get_default_right_vec();
        let expected_vec = get_sorted_expected_merge_value();

        let mut merge_vec = merge_right_vars(left_vec, right_vec);

        merge_vec.sort_by(|a, b| a.key.cmp(&b.key));

        assert_eq!(merge_vec, expected_vec)
    }
}
