use std::collections::{BTreeMap, HashSet};
use std::str::FromStr;

use convert_case::{Case, Casing};
use mime::Mime;
use oas3::spec::{ObjectOrReference, PathItem, SchemaType, SchemaTypeSet};
use oas3::{OpenApiV3Spec, Schema};

use crate::core::config::{Arg, Config, Field, Http, KeyValue, Type};
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

///
/// The TypeName enum represents the name of a type in the generated code.
/// Creating a special type is required since the types can be recursive
#[derive(Debug)]
enum TypeName {
    ListOf(Box<TypeName>),
    Name(String),
}

impl TypeName {
    fn name(&self) -> Option<String> {
        match self {
            TypeName::ListOf(_) => None,
            TypeName::Name(name) => Some(name.clone()),
        }
    }

    fn into_tuple(self) -> (bool, String) {
        match self {
            TypeName::ListOf(inner) => (true, inner.name().unwrap()),
            TypeName::Name(name) => (false, name),
        }
    }
}

fn name_from_ref_path<T>(obj_or_ref: &ObjectOrReference<T>) -> Option<String> {
    match obj_or_ref {
        ObjectOrReference::Ref { ref_path } => {
            ref_path.split('/').last().map(|a| a.to_case(Case::Pascal))
        }
        ObjectOrReference::Object(_) => None,
    }
}

fn schema_type_to_string(typ: &SchemaTypeSet) -> String {
    match typ {
        SchemaTypeSet::Single(
            x @ (SchemaType::Boolean | SchemaType::String | SchemaType::Array | SchemaType::Object),
        ) => {
            format!("{x:?}")
        }
        SchemaTypeSet::Single(SchemaType::Integer | SchemaType::Number) => "Int".into(),
        x => unreachable!("encountered: {x:?}"),
    }
}

fn schema_to_primitive_type(typ: &SchemaTypeSet) -> Option<String> {
    match typ {
        SchemaTypeSet::Single(SchemaType::Array | SchemaType::Object) => None,
        x @ SchemaTypeSet::Single(_) => Some(schema_type_to_string(x)),
        _ => unreachable!(),
    }
}

fn can_define_type(schema: &Schema) -> bool {
    !schema.properties.is_empty()
        || !schema.all_of.is_empty()
        || !schema.any_of.is_empty()
        || !schema.one_of.is_empty()
        || !schema.enum_values.is_empty()
}

fn unknown_type() -> String {
    "Unknown".to_string()
}

impl<'a> SingleQueryGenerator<'a> {
    fn get_schema_type(&self, schema: Schema, name: Option<String>) -> anyhow::Result<TypeName> {
        Ok(if let Some(element) = schema.items {
            let inner_schema = element.resolve(self.spec)?;
            if inner_schema.schema_type == Some(SchemaTypeSet::Single(SchemaType::String))
                && !inner_schema.enum_values.is_empty()
            {
                TypeName::ListOf(Box::new(TypeName::Name(unknown_type())))
            } else if let Some(name) = name_from_ref_path(element.as_ref())
                .or_else(|| schema_to_primitive_type(inner_schema.schema_type.as_ref()?))
            {
                TypeName::ListOf(Box::new(TypeName::Name(name)))
            } else {
                TypeName::ListOf(Box::new(self.get_schema_type(inner_schema, None)?))
            }
        } else if schema.schema_type == Some(SchemaTypeSet::Single(SchemaType::String))
            && !schema.enum_values.is_empty()
        {
            TypeName::Name(unknown_type())
        } else if let Some(
            typ @ SchemaTypeSet::Single(
                SchemaType::Integer | SchemaType::String | SchemaType::Number | SchemaType::Boolean,
            ),
        ) = schema.schema_type
        {
            TypeName::Name(schema_type_to_string(&typ))
        } else if let Some(name) = name {
            TypeName::Name(name)
        } else if can_define_type(&schema) {
            TypeName::Name(unknown_type())
        } else {
            TypeName::Name("JSON".to_string())
        })
    }
}

impl<'a> Transform for SingleQueryGenerator<'a> {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
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
            let Some((_, first_response)) = operation
                .responses
                .as_ref()
                .and_then(|responses| responses.first_key_value())
            else {
                return Valid::fail(format!("skipping {path}: no sample response found"));
            };
            let Ok(response) = first_response.resolve(self.spec) else {
                return Valid::fail(format!("skipping {path}: no sample response found"));
            };

            let Some(Ok(output_type)) =
                response
                    .content
                    .first_key_value()
                    .and_then(|(content_type, v)| {
                        let mime = Mime::from_str(content_type.as_str()).unwrap();
                        if mime.eq(&mime::TEXT_PLAIN) {
                            Some(Ok(TypeName::Name("String".to_string())))
                        } else {
                            let obj_or_ref = v.schema.as_ref()?;
                            Some(
                                obj_or_ref
                                    .resolve(self.spec)
                                    .map_err(|err| err.to_string())
                                    .and_then(|schema| {
                                        self.get_schema_type(schema, name_from_ref_path(obj_or_ref))
                                            .map_err(|err| err.to_string())
                                    }),
                            )
                        }
                    })
            else {
                return Valid::fail(format!("skipping {path}: unable to detect output type"));
            };

            let (is_list, name) = output_type.into_tuple();

            let args = Valid::from_iter::<(String, Arg)>(operation.parameters.iter(), |param| {
                let result = param
                    .resolve(self.spec)
                    .map_err(|err| err.to_string())
                    .and_then(|param| {
                        self.get_schema_type(param.schema.clone().unwrap(), None)
                            .map_err(|err| err.to_string())
                            .map(TypeName::into_tuple)
                            .map(|type_tuple| (param, type_tuple))
                    })
                    .map(|(param, (is_list, name))| {
                        (
                            param.name.to_case(Case::Camel),
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
            Valid::succeed(config)
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
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        config.types.insert(self.query.to_string(), Type::default());
        let path_iter = self
            .spec
            .paths
            .clone()
            .into_iter()
            .flat_map(|x| x.into_iter());

        let result = Valid::from_iter(path_iter, |(path, path_item)| {
            SingleQueryGenerator {
                query: self.query,
                path,
                path_item,
                spec: self.spec,
                base_url: self.base_url.clone(),
            }
            .transform(config.clone())
            .map(|new_config| {
                config = new_config;
            })
        });

        if let Err(err) = result.to_result() {
            tracing::debug!("Config generation encountered following errors: {err:?}");
        }

        Valid::succeed(config)
    }
}
