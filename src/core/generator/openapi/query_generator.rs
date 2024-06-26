use convert_case::{Case, Casing};
use oas3::spec::{ObjectOrReference, PathItem};
use oas3::OpenApiV3Spec;

use crate::core::config::{Config, Field, Http, Type};
use crate::core::http::Method;
use crate::core::transform::Transform;
use crate::core::valid::{Valid, Validator};

struct SingleQueryGenerator<'a> {
    query: &'a str,
    path: String,
    path_item: PathItem,
    spec: &'a OpenApiV3Spec,
}

fn name_from_ref_path<T>(obj_or_ref: &ObjectOrReference<T>) -> Option<String> {
    match obj_or_ref {
        ObjectOrReference::Ref { ref_path } => {
            ref_path.split('/').last().map(|a| a.to_case(Case::Pascal))
        }
        ObjectOrReference::Object(_) => None,
    }
}

impl<'a> Transform for SingleQueryGenerator<'a> {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        let path = self.path.clone();
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

            match name_from_ref_path(&output_type) {
                Some(type_of) => {
                    let field = Field {
                        type_of,
                        http: Some(Http { path, method, ..Default::default() }),
                        doc: operation.description,
                        ..Default::default()
                    };

                    config.types.get_mut(self.query).map(|typ| {
                        typ.fields
                            .insert(operation.operation_id.unwrap().to_case(Case::Camel), field)
                    });
                    Valid::succeed(config)
                }
                None => {
                    Valid::fail(format!(
                        "skipping {path}: unable to find name of the type"
                    ))
                }
            }
        })
    }
}

pub struct QueryGenerator<'a> {
    query: &'a str,
    spec: &'a OpenApiV3Spec,
}

impl<'a> QueryGenerator<'a> {
    pub fn new(query: &'a str, spec: &'a OpenApiV3Spec) -> Self {
        Self { query, spec }
    }
}

impl<'a> Transform for QueryGenerator<'a> {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        config.types.insert(self.query.to_string(), Type::default());
        let path_iter = self.spec.paths.clone().into_iter();

        Valid::from_iter(path_iter, |(path, path_item)| {
            SingleQueryGenerator { query: self.query, path, path_item, spec: self.spec }
                .transform(config.clone())
                .map(|new_config| {
                    config = new_config;
                })
        });

        Valid::succeed(config)
    }
}
