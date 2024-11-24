use tailcall_valid::{Valid, ValidationError, Validator};

use crate::core::blueprint::*;
use crate::core::config::group_by::GroupBy;
use crate::core::config::{Field, Resolver};
use crate::core::endpoint::Endpoint;
use crate::core::http::{HttpFilter, Method, RequestTemplate};
use crate::core::ir::model::{IO, IR};
use crate::core::mustache::Segment;
use crate::core::scalar::Scalar;
use crate::core::try_fold::TryFold;
use crate::core::{config, helpers, Mustache};

fn path_validator<'a>(
    module: &ConfigModule,
    mut path_iter: impl Iterator<Item = &'a String>,
    type_of: &str,
) -> Valid<(), String> {
    match module.find_type(type_of) {
        Some(type_def) => match path_iter.next() {
            Some(arg) => match type_def.fields.get(arg) {
                Some(field_type) => path_validator(module, path_iter, field_type.type_of.name()),
                None => Valid::fail(format!("Field '{}' not found in type '{}'.", arg, type_of)),
            },
            None => Valid::fail(format!("Type '{}' is not a scalar type.", type_of)),
        },
        None if Scalar::is_predefined(type_of) || module.find_enum(type_of).is_some() => {
            Valid::succeed(())
        }
        None => Valid::fail(format!("Type '{}' not found in config.", type_of)),
    }
}

/// Function to validate the arguments in the HTTP resolver.
fn validate_arg(
    config_module: &config::ConfigModule,
    template: Mustache,
    field: &Field,
    field_name: Option<&str>,
) -> Valid<(), String> {
    let field_name = field_name.unwrap_or_default();
    Valid::from_iter(template.segments(), |segment| match segment {
        Segment::Expression(expr) if expr.first().map_or(false, |v| v.contains("args")) => {
            match expr.get(1) {
                Some(arg_name) if field.args.get(arg_name).is_some() => {
                    let arg_type_of = field.args.get(arg_name).as_ref().unwrap().type_of.name();
                    path_validator(config_module, expr.iter().skip(2), arg_type_of).trace(arg_name)
                }
                Some(arg_name) => {
                    let message = if !field_name.is_empty() {
                        format!(
                            "Argument '{}' not found in field '{}'.",
                            arg_name, field_name
                        )
                    } else {
                        format!("Argument '{}' not found.", arg_name)
                    };
                    Valid::fail(message).trace(arg_name)
                }
                None => {
                    let message = if !field_name.is_empty() {
                        format!("Invalid Argument defined in field '{}'.", field_name)
                    } else {
                        "Invalid Argument defined in field.".to_string()
                    };
                    Valid::fail(message)
                }
            }
        }
        _ => Valid::succeed(()),
    })
    .unit()
}

pub fn compile_http(
    config_module: &config::ConfigModule,
    http: &config::Http,
    field: &Field,
    field_name: Option<&str>,
) -> Valid<IR, String> {
    let is_list = field.type_of.is_list();
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
        .and(
            Valid::from_iter(http.query.iter(), |query| {
                validate_arg(
                    config_module,
                    Mustache::parse(query.value.as_str()),
                    field,
                    field_name,
                )
            })
            .unit()
            .trace("query"),
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
        |(config_module, field, type_of, field_name), b_field| {
            let Some(Resolver::Http(http)) = &field.resolver else {
                return Valid::succeed(b_field);
            };

            compile_http(config_module, http, field, Some(field_name))
                .map(|resolver| b_field.resolver(Some(resolver)))
                .and_then(|b_field| {
                    b_field
                        .validate_field(type_of, config_module)
                        .map_to(b_field)
                })
        },
    )
}
