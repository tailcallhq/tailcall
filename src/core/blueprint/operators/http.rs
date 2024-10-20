use crate::core::blueprint::*;
use crate::core::config::group_by::GroupBy;
use crate::core::config::{Field, Resolver};
use crate::core::endpoint::Endpoint;
use crate::core::http::{HttpFilter, Method, RequestTemplate};
use crate::core::ir::model::{IO, IR};
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, ValidationError, Validator};
use crate::core::{config, helpers, Mustache};

pub fn compile_http(
    config_module: &config::ConfigModule,
    http: &config::Http,
    is_list: bool,
) -> Valid<IR, String> {
    let dedupe = http.dedupe.unwrap_or_default();

    Valid::<(), String>::fail("GroupBy is only supported for GET requests".to_string())
        .when(|| !http.batch_key.is_empty() && http.method != Method::GET)
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
        .and(Valid::succeed(
            http.url
                .as_str()
        ))
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

            let io = if !http.batch_key.is_empty() && http.method == Method::GET {
                // Find a query parameter that contains a reference to the {{.value}} key
                let key = http.query.iter().find_map(|q| {
                    Mustache::parse(&q.value)
                        .expression_contains("value")
                        .then(|| q.key.clone())
                });
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
