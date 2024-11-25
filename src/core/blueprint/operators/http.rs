use regex::Regex;
use tailcall_valid::{Valid, ValidationError, Validator};

use crate::core::blueprint::*;
use crate::core::config::group_by::GroupBy;
use crate::core::config::{Field, Resolver};
use crate::core::endpoint::Endpoint;
use crate::core::http::{HttpFilter, Method, RequestTemplate};
use crate::core::ir::model::{IO, IR};
use crate::core::try_fold::TryFold;
use crate::core::{config, helpers, Mustache};

pub fn compile_http(
    config_module: &config::ConfigModule,
    http: &config::Http,
    is_list: bool,
) -> Valid<IR, String> {
    let dedupe = http.dedupe.unwrap_or_default();

    Valid::<(), String>::fail("GroupBy is only supported for GET/POST requests".to_string())
        .when(|| !http.batch_key.is_empty() && !matches!(http.method, Method::GET | Method::POST))
        .and(
            Valid::<(), String>::fail(
                "Batching capability was used without enabling it in upstream".to_string(),
            )
            .when(|| {
                (config_module.upstream.get_delay() < 1
                    || config_module.upstream.get_max_size() < 1)
                    && !http.batch_key.is_empty()
            }),
        )
        .and(
            Valid::<(), String>::fail(
                "Only one dynamic key allowed in POST batch request.".to_string(),
            )
            .when(|| {
                if http.method != Method::POST {
                    return false;
                }

                if !http.batch_key.is_empty() {
                    let is_single_key = http
                        .body
                        .as_ref()
                        .map(|b| extract_expression_keys(b).len() == 1)
                        .unwrap_or_default();

                    return !is_single_key;
                }

                false
            })
            .trace("body"),
        )
        .and(Valid::succeed(http.url.as_str()))
        .zip(helpers::headers::to_mustache_headers(&http.headers))
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

            RequestTemplate::try_from(
                Endpoint::new(base_url.to_string())
                    .method(http.method.clone())
                    .query(query)
                    .body(http.body.clone())
                    .encoding(http.encoding.clone()),
            )
            .map(|req_tmpl| req_tmpl.headers(headers))
            .map_err(|e| ValidationError::new(e.to_string()))
            .into()
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
                    // find the key from the body where value is mustache template.
                    http.body
                        .as_ref()
                        .map(|b| extract_expression_keys(b))
                        .and_then(|keys| {
                            if keys.len() == 1 {
                                Some(keys[0].clone())
                            } else {
                                None
                            }
                        })
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

pub fn update_http<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
        |(config_module, field, type_of, _), b_field| {
            let Some(Resolver::Http(http)) = &field.resolver else {
                return Valid::succeed(b_field);
            };

            compile_http(config_module, http, field.type_of.is_list())
                .map(|resolver| b_field.resolver(Some(resolver)))
                .and_then(|b_field| {
                    b_field
                        .validate_field(type_of, config_module)
                        .map_to(b_field)
                })
        },
    )
}

/// extracts the keys from the json representation, if the value is of mustache
/// template type.
fn extract_expression_keys(json: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let re = Regex::new(r#""([^"]+)"\s*:\s*"\{\{.*?\}\}""#).unwrap();
    for cap in re.captures_iter(json) {
        if let Some(key) = cap.get(1) {
            keys.push(key.as_str().to_string());
        }
    }
    keys
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extract_expression_keys_from_str() {
        let json = r#"{"body":"d","userId":"{{.value.uid}}","nested":{"other":"{{test}}"}}"#;
        let keys = extract_expression_keys(json);
        assert_eq!(keys, vec!["userId", "other"]);
    }

    #[test]
    fn test_with_non_json_value() {
        let json = r#"{{.value}}"#;
        let keys = extract_expression_keys(json);
        assert_eq!(keys, Vec::<String>::new());
    }
}
