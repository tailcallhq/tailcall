use crate::blueprint::*;
use crate::config::group_by::GroupBy;
use crate::config::Field;
use crate::endpoint::Endpoint;
use crate::http::{Method, RequestTemplate};
use crate::lambda::{Expression, Lambda, IO};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError, Validator};
use crate::{config, helpers};

pub fn compile_http(
    config_set: &config::ConfigSet,
    field: &config::Field,
    http: &config::Http,
) -> Valid<Expression, String> {
    Valid::<(), String>::fail("GroupBy is only supported for GET requests".to_string())
        .when(|| !http.group_by.is_empty() && http.method != Method::GET)
        .and(
            Valid::<(), String>::fail(
                "GroupBy can only be applied if batching is enabled".to_string(),
            )
            .when(|| {
                (config_set.upstream.get_delay() < 1 || config_set.upstream.get_max_size() < 1)
                    && !http.group_by.is_empty()
            }),
        )
        .and(Valid::from_option(
            http.base_url
                .as_ref()
                .or(config_set.upstream.base_url.as_ref()),
            "No base URL defined".to_string(),
        ))
        .zip(helpers::headers::to_mustache_headers(&http.headers))
        .and_then(|(base_url, headers)| {
            let mut base_url = base_url.trim_end_matches('/').to_owned();
            base_url.push_str(http.path.clone().as_str());

            let query = http
                .query
                .clone()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let output_schema = to_json_schema_for_field(field, config_set);
            let input_schema = to_json_schema_for_args(&field.args, config_set);

            RequestTemplate::try_from(
                Endpoint::new(base_url.to_string())
                    .method(http.method.clone())
                    .query(query)
                    .output(output_schema)
                    .input(input_schema)
                    .body(http.body.clone())
                    .encoding(http.encoding.clone()),
            )
            .map(|req_tmpl| req_tmpl.headers(headers))
            .map_err(|e| ValidationError::new(e.to_string()))
            .into()
        })
        .map(|req_template| {
            if !http.group_by.is_empty() && http.method == Method::GET {
                Expression::IO(IO::Http {
                    req_template,
                    group_by: Some(GroupBy::new(http.group_by.clone())),
                    dl_id: None,
                })
            } else {
                Lambda::from_request_template(req_template).expression
            }
        })
}

pub fn update_http<'a>(
) -> TryFold<'a, (&'a ConfigSet, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
    TryFold::<(&ConfigSet, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
        |(config_set, field, type_of, _), b_field| {
            let Some(http) = &field.http else {
                return Valid::succeed(b_field);
            };

            compile_http(config_set, field, http)
                .map(|resolver| b_field.resolver(Some(resolver)))
                .and_then(|b_field| b_field.validate_field(type_of, config_set).map_to(b_field))
        },
    )
}
