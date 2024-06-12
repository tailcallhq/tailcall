use crate::core::blueprint::*;
use crate::core::config::group_by::GroupBy;
use crate::core::config::position::Pos;
use crate::core::config::Field;
use crate::core::endpoint::Endpoint;
use crate::core::http::{HttpFilter, Method, RequestTemplate};
use crate::core::ir::{IO, IR};
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, ValidationError, Validator};
use crate::core::{config, helpers};

pub fn compile_http(
    config_module: &config::ConfigModule,
    http: &Pos<config::Http>,
) -> Valid<IR, String> {
    Valid::<(), String>::fail("GroupBy is only supported for GET requests".to_string())
        .when(|| !http.group_by.is_empty() && http.method.inner != Method::GET)
        .and(
            Valid::<(), String>::fail(
                "GroupBy can only be applied if batching is enabled".to_string(),
            )
            .when(|| {
                (config_module.upstream.get_delay() < 1
                    || config_module.upstream.get_max_size() < 1)
                    && !http.group_by.is_empty()
            }),
        )
        .and(Valid::from_option(
            http.base_url
                .as_ref()
                .or(config_module.upstream.base_url.as_ref()),
            "No base URL defined".to_string(),
        ))
        .zip(helpers::headers::to_mustache_headers(
            &http
                .headers
                .iter()
                .map(|header| header.inner.to_owned())
                .collect::<Vec<_>>(),
        ))
        .and_then(|(base_url, headers)| {
            let mut base_url = base_url.trim_end_matches('/').to_owned();
            base_url.push_str(http.path.clone().as_str());

            let query = http
                .query
                .clone()
                .iter()
                .map(|key_value| (key_value.key.clone(), key_value.value.clone()))
                .collect();

            RequestTemplate::try_from(
                Endpoint::new(base_url.to_string())
                    .method(http.method.inner.clone())
                    .query(query)
                    .body(http.body.to_owned().map(|body| body.inner))
                    .encoding(http.encoding.inner.clone()),
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
                .map(|on_request| HttpFilter { on_request: on_request.inner });

            if !http.group_by.is_empty() && http.method.inner == Method::GET {
                IR::IO(IO::Http {
                    req_template,
                    group_by: Some(GroupBy::new(http.group_by.clone().inner)),
                    dl_id: None,
                    http_filter,
                })
            } else {
                IR::IO(IO::Http { req_template, group_by: None, dl_id: None, http_filter })
            }
        })
}

pub fn update_http<'a>() -> TryFold<
    'a,
    (
        &'a ConfigModule,
        &'a Pos<Field>,
        &'a Pos<config::Type>,
        &'a str,
    ),
    FieldDefinition,
    String,
> {
    TryFold::<(&ConfigModule, &Pos<Field>, &Pos<config::Type>, &'a str), FieldDefinition, String>::new(
        |(config_module, field, type_of, _), b_field| {
            let Some(http) = &field.http.as_ref() else {
                return Valid::succeed(b_field);
            };

            compile_http(config_module, http)
                .map(|resolver| b_field.resolver(Some(resolver)))
                .and_then(|b_field| {
                    b_field
                        .validate_field(type_of, config_module)
                        .map_to(b_field)
                })
        },
    )
}
