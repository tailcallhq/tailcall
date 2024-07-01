use std::collections::{BTreeMap, HashMap, HashSet};

use convert_case::{Case, Casing};
use oas3::spec::PathItem;
use oas3::{OpenApiV3Spec, Schema};

use crate::core::config::{Arg, Config, Field, Http, KeyValue, Type};
use crate::core::generator::openapi::helpers::{get_schema_type, name_from_ref_path, TypeName};
use crate::core::http::Method;
use crate::core::transform::Transform;
use crate::core::valid::{Valid, Validator};

struct SingleQueryGenerator<'a> {
    query: &'a str,
    path: String,
    path_item: PathItem,
    spec: &'a OpenApiV3Spec,
    base_url: Option<String>,
}

impl<'a> Transform for SingleQueryGenerator<'a> {
    type Value = (HashMap<String, Schema>, Config);
    type Error = String;

    fn transform(&self, (mut types, mut config): Self::Value) -> Valid<Self::Value, Self::Error> {
        let mut path = self.path.clone();
        let path_item = self.path_item.clone();

        let method_and_operation = [
            (Method::GET, path_item.get),
            (Method::HEAD, path_item.head),
            (Method::OPTIONS, path_item.options),
            (Method::TRACE, path_item.trace),
            (Method::PUT, path_item.put),
            (Method::POST, path_item.post),
            (Method::DELETE, path_item.delete),
            (Method::PATCH, path_item.patch),
        ]
        .into_iter()
        .filter_map(|(method, operation)| operation.map(|operation| (method, operation)))
        .next();

        Valid::from_option(
            method_and_operation,
            format!("skipping {path}: no operation found"),
        )
        .and_then(|(method, operation)| {
            let Some((_, first_response)) = operation.responses.first_key_value() else {
                return Valid::fail(format!("skipping {path}: no sample response found"));
            };
            let Ok(response) = first_response.resolve(self.spec) else {
                return Valid::fail(format!("skipping {path}: no sample response found"));
            };

            let Some(output_type) = response
                .content
                .first_key_value()
                .map(|(_, v)| v)
                .cloned()
                .and_then(|v| v.schema)
            else {
                return Valid::fail(format!("skipping {path}: unable to detect output type"));
            };

            let args = Valid::from_iter::<(String, Arg)>(operation.parameters.iter(), |param| {
                let result = param
                    .resolve(self.spec)
                    .map_err(|err| err.to_string())
                    .and_then(|param| {
                        get_schema_type(
                            self.spec,
                            param.schema.clone().unwrap(),
                            param.param_type.clone(),
                            &mut types,
                        )
                        .map_err(|err| err.to_string())
                        .map(TypeName::into_tuple)
                        .map(|type_tuple| (param, type_tuple))
                    })
                    .map(|(param, (is_list, name))| {
                        (
                            param.name,
                            Arg {
                                type_of: name,
                                list: is_list,
                                required: param.required.unwrap_or_default(),
                                doc: None,
                                modify: None,
                                default_value: None,
                            },
                        )
                    });

                match result {
                    Ok(arg) => Valid::succeed(arg),
                    Err(err) => Valid::fail(err),
                }
            });

            let args: BTreeMap<String, Arg> = match args.to_result() {
                Ok(args) => args.into_iter().collect(),
                Err(err) => return Valid::fail(err.to_string()),
            };

            let type_tuple = output_type
                .resolve(self.spec)
                .map_err(|err| err.to_string())
                .and_then(|schema| {
                    get_schema_type(
                        self.spec,
                        schema,
                        name_from_ref_path(&output_type),
                        &mut types,
                    )
                    .map_err(|err| err.to_string())
                })
                .map(TypeName::into_tuple);

            let (is_list, name) = match type_tuple {
                Ok((is_list, name)) => (is_list, name),
                Err(err) => return Valid::fail(err.to_string()),
            };

            let mut url_params = HashSet::new();
            if !args.is_empty() {
                let re = regex::Regex::new(r"\{\w+\}").unwrap();
                path = re
                    .replacen(path.as_str(), 0, |cap: &regex::Captures| {
                        let arg_name = &cap[0][1..cap[0].len() - 1];
                        url_params.insert(arg_name.to_string());
                        format!("{{{{.args.{}}}}}", arg_name)
                    })
                    .to_string();
            }

            let query_params = args
                .iter()
                .filter(|&(key, _)| !url_params.contains(key))
                .map(|(key, _)| KeyValue {
                    key: key.to_string(),
                    value: format!("{{{{.args.{}}}}}", key),
                })
                .collect();

            let field = Field {
                type_of: name,
                list: is_list,
                args,
                http: Some(Http {
                    path,
                    base_url: self.base_url.clone(),
                    method,
                    query: query_params,
                    ..Default::default()
                }),
                doc: operation.description,
                ..Default::default()
            };

            config.types.get_mut(self.query).map(|typ| {
                typ.fields
                    .insert(operation.operation_id.unwrap().to_case(Case::Camel), field)
            });
            Valid::succeed((types, config))
        })
    }
}

pub struct QueryGenerator<'a> {
    query: &'a str,
    spec: &'a OpenApiV3Spec,
    base_url: Option<String>,
}

impl<'a> QueryGenerator<'a> {
    pub fn new(query: &'a str, spec: &'a OpenApiV3Spec) -> Self {
        let base_url = spec.servers.first().map(|server| server.url.clone());
        Self { query, spec, base_url }
    }
}

impl<'a> Transform for QueryGenerator<'a> {
    type Value = (HashMap<String, Schema>, Config);
    type Error = String;

    fn transform(&self, (mut types, mut config): Self::Value) -> Valid<Self::Value, Self::Error> {
        config.types.insert(self.query.to_string(), Type::default());
        let path_iter = self.spec.paths.clone().into_iter();

        Valid::from_iter(path_iter, |(path, path_item)| {
            SingleQueryGenerator {
                query: self.query,
                path,
                path_item,
                spec: self.spec,
                base_url: self.base_url.clone(),
            }
            .transform((types.clone(), config.clone()))
            .map(|(new_types, new_config)| {
                types = new_types;
                config = new_config;
            })
        });

        Valid::succeed((types, config))
    }
}
