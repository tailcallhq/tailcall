use std::slice::Iter;

use crate::core::blueprint::*;
use crate::core::config::group_by::GroupBy;
use crate::core::config::{Field, Resolver};
use crate::core::endpoint::Endpoint;
use crate::core::http::{HttpFilter, Method, RequestTemplate};
use crate::core::ir::model::{IO, IR};
use crate::core::mustache::Segment;
use crate::core::scalar::Scalar;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, ValidationError, Validator};
use crate::core::{config, helpers, Mustache};

fn check_ty(mut iter: Iter<String>, module: &ConfigModule, cur_ty: &str) -> bool {
    let type_ = module.types.get(cur_ty);
    if type_.is_none() {
        return Scalar::is_predefined(cur_ty) || module.find_enum(cur_ty).is_some();
    }
    let type_ = type_.unwrap();

    let cur = iter.next();
    if cur.is_none() {
        return Scalar::is_predefined(cur_ty) || module.find_enum(cur_ty).is_some();
    }

    let cur = cur.unwrap();
    if type_.fields.contains_key(cur) {
        let next = type_.fields.get(cur).unwrap().type_of.name();
        check_ty(iter, module, next)
    } else {
        false
    }
}

fn check_args(mut iter: Iter<String>, module: &ConfigModule, field: &Field) -> bool {
    let cur = iter.next();
    if cur.is_none() {
        return Scalar::is_predefined(field.type_of.name())
            || module.find_enum(field.type_of.name()).is_some();
    }
    let cur = cur.unwrap();
    field.args.contains_key(cur)
        && check_ty(
            iter,
            module,
            field.args.get(cur).as_ref().unwrap().type_of.name(),
        )
}

fn check_scalar(value: &Mustache, module: &ConfigModule, field: &Field) -> bool {
    let mut ans = true;
    for segment in value.segments() {
        match segment {
            Segment::Literal(_) => {}
            Segment::Expression(value) => {
                if !value.is_empty() && value[0].as_str() == "args" {
                    ans = check_args(value[1..].iter().clone(), module, field);
                }
            }
        }
    }
    ans
}

pub fn compile_http(
    config_module: &config::ConfigModule,
    http: &config::Http,
    field: &Field,
    is_federation: bool,
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
        .and(Valid::succeed(http.url.as_str()))
        .zip(helpers::headers::to_mustache_headers(&http.headers))
        .and_then(|(base_url, headers)| {
            Valid::from_iter(http.query.clone(), |key_value| {
                let mustache = Mustache::parse(key_value.value.as_str());
                let mut ans = Valid::succeed((
                    key_value.key,
                    key_value.value,
                    key_value.skip_empty.unwrap_or_default(),
                ));
                if !is_federation && !check_scalar(&mustache, config_module, field) {
                    ans = Valid::fail("Query parameter must be a scalar".to_string());
                }

                ans
            })
            .and_then(|query| Valid::succeed((base_url, headers, query)))
        })
        .and_then(|(base_url, headers, query)| {
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

            compile_http(config_module, http, field, false)
                .map(|resolver| b_field.resolver(Some(resolver)))
                .and_then(|b_field| {
                    b_field
                        .validate_field(type_of, config_module)
                        .map_to(b_field)
                })
        },
    )
}
