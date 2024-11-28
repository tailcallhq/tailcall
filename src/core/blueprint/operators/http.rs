use tailcall_valid::{Valid, Validator};
use template_validation::validate_argument;

use crate::core::blueprint::*;
use crate::core::config::group_by::GroupBy;
use crate::core::config::Field;
use crate::core::endpoint::Endpoint;
use crate::core::http::{Method, RequestTemplate};
use crate::core::ir::model::{IO, IR};
use crate::core::js_hooks::JsHooks;
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

    Valid::<(), BlueprintError>::fail(BlueprintError::GroupByOnlyForGet)
        .when(|| !http.batch_key.is_empty() && http.method != Method::GET)
        .and(
            Valid::<(), BlueprintError>::fail(BlueprintError::IncorrectBatchingUsage).when(|| {
                (config_module.upstream.get_delay() < 1
                    || config_module.upstream.get_max_size() < 1)
                    && !http.batch_key.is_empty()
            }),
        )
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
        .map(|req_template| {
            // marge http and upstream on_request
            let on_request = http
                .on_request
                .clone()
                .or(config_module.upstream.on_request.clone());
            let on_response_body = http.on_response_body.clone();
            let hook = JsHooks::try_new(on_request, on_response_body).ok();

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
                    is_list,
                    dedupe,
                    hook,
                })
            } else {
                IR::IO(IO::Http {
                    req_template,
                    group_by: None,
                    dl_id: None,
                    is_list,
                    dedupe,
                    hook,
                })
            };
            (io, &http.select)
        })
        .and_then(apply_select)
}
