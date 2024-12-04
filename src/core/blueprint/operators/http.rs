use std::borrow::Cow;

use tailcall_valid::{Valid, Validator};
use template_validation::validate_argument;

use crate::core::blueprint::*;
use crate::core::config::group_by::GroupBy;
use crate::core::config::Field;
use crate::core::endpoint::Endpoint;
use crate::core::http::{HttpFilter, Method, RequestTemplate};
use crate::core::ir::model::{IO, IR};
use crate::core::{config, helpers, Mustache};

pub fn compile_http(
    config_module: &config::ConfigModule,
    http: &config::Http,
    field: &Field,
) -> Valid<IR, BlueprintError> {
    let is_list = field.type_of.is_list();
    let dedupe = http.dedupe.unwrap_or_default();
    let mustache_headers = match helpers::headers::to_mustache_headers(&http.headers).to_result() {
        Ok(mustache_headers) => Valid::succeed(mustache_headers),
        Err(e) => Valid::from_validation_err(BlueprintError::from_validation_string(e)),
    };

    Valid::<(), BlueprintError>::fail(BlueprintError::IncorrectBatchingUsage)
        .when(|| {
            (config_module.upstream.get_delay() < 1 || config_module.upstream.get_max_size() < 1)
                && !http.batch_key.is_empty()
        })
        .and(
            Valid::from_iter(http.query.iter(), |query| {
                validate_argument(config_module, Mustache::parse(query.value.as_str()), field)
            })
            .unit()
            .trace("query"),
        )
        .and(Valid::succeed(http.url.as_str()))
        .zip(mustache_headers)
        .and_then(|(base_url, headers)| {
            let query = http
                .query
                .clone()
                .iter()
                .map(|key_value| {
                    (
                        key_value.key.clone(),
                        key_value.value.clone(),
                        key_value.skip_empty.unwrap_or_default(),
                    )
                })
                .collect();

            match RequestTemplate::try_from(
                Endpoint::new(base_url.to_string())
                    .method(http.method.clone())
                    .query(query)
                    .body(http.body.clone())
                    .encoding(http.encoding.clone()),
            )
            .map(|req_tmpl| req_tmpl.headers(headers))
            {
                Ok(data) => Valid::succeed(data),
                Err(e) => Valid::fail(BlueprintError::Error(e)),
            }
        })
        .and_then(|request_template| {
            if !http.batch_key.is_empty() && (http.body.is_some() || http.method != Method::GET) {
                let keys = http.body.as_ref().map(|b| extract_expression_paths(b));
                if let Some(keys) = keys {
                    // only one dynamic value allowed in body for batching to work.
                    if keys.len() != 1 {
                        Valid::fail(BlueprintError::BatchRequiresDynamicParameter).trace("body")
                    } else {
                        Valid::succeed(request_template)
                    }
                } else {
                    Valid::fail(BlueprintError::BatchRequiresDynamicParameter).trace("body")
                }
            } else {
                Valid::succeed(request_template)
            }
        })
        .map(|req_template| {
            // marge http and upstream on_request
            let http_filter = http
                .on_request
                .clone()
                .or(config_module.upstream.on_request.clone())
                .map(|on_request| HttpFilter { on_request });

            let group_by_clause = !http.batch_key.is_empty()
                && (http.method == Method::GET || http.method == Method::POST);
            let io = if group_by_clause {
                // Find a query parameter that contains a reference to the {{.value}} key
                let key = if http.method == Method::GET {
                    http.query.iter().find_map(|q| {
                        Mustache::parse(&q.value)
                            .expression_contains("value")
                            .then(|| q.key.clone())
                    })
                } else {
                    None
                };

                IR::IO(IO::Http {
                    req_template,
                    group_by: Some(GroupBy::new(http.batch_key.clone(), key)),
                    dl_id: None,
                    http_filter,
                    is_list,
                    dedupe,
                })
            } else {
                IR::IO(IO::Http {
                    req_template,
                    group_by: None,
                    dl_id: None,
                    http_filter,
                    is_list,
                    dedupe,
                })
            };
            (io, &http.select)
        })
        .and_then(apply_select)
}

/// extracts the keys from the json representation, if the value is of mustache
/// template type.
fn extract_expression_paths(json: &serde_json::Value) -> Vec<Vec<Cow<'_, str>>> {
    fn extract_paths<'a>(
        json: &'a serde_json::Value,
        path: &mut Vec<Cow<'a, str>>,
    ) -> Vec<Vec<Cow<'a, str>>> {
        let mut keys = vec![];
        match json {
            serde_json::Value::Array(arr) => {
                arr.iter().enumerate().for_each(|(idx, v)| {
                    let idx = idx.to_string();
                    path.push(Cow::Owned(idx));
                    keys.extend(extract_paths(v, path));
                });
            }
            serde_json::Value::Object(obj) => {
                obj.iter().for_each(|(k, v)| {
                    path.push(Cow::Borrowed(k));
                    keys.extend(extract_paths(v, path));
                    path.pop();
                });
            }
            serde_json::Value::String(s) => {
                if !Mustache::parse(s).is_const() {
                    keys.push(path.to_vec());
                }
            }
            _ => {}
        }
        keys
    }

    extract_paths(json, &mut Vec::new())
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_extract_expression_keys_from_nested_objects() {
        let json = r#"{"body":"d","userId":"{{.value.uid}}","nested":{"other":"{{test}}"}}"#;
        let json = serde_json::from_str(json).unwrap();
        let keys = extract_expression_paths(&json);
        assert_eq!(keys.len(), 2);
        assert_eq!(keys, vec![vec!["userId"], vec!["nested", "other"]]);
    }

    #[test]
    fn test_extract_expression_keys_from_mixed_json() {
        let json = r#"{"body":"d","userId":"{{.value.uid}}","nested":{"other":"{{test}}"},"meta":[{"key": "id", "value": "{{.value.userId}}"}]}"#;
        let json = serde_json::from_str(json).unwrap();
        let keys = extract_expression_paths(&json);
        assert_eq!(keys.len(), 3);
        assert_eq!(
            keys,
            vec![
                vec!["userId"],
                vec!["nested", "other"],
                vec!["meta", "0", "value"]
            ]
        );
    }

    #[test]
    fn test_with_non_json_value() {
        let json = json!(r#"{{.value}}"#);
        let keys = extract_expression_paths(&json);
        assert!(keys.iter().all(|f| f.is_empty()));
    }
}
