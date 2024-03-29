use std::collections::BTreeMap;

use oas3::spec::ObjectOrReference;
use oas3::{Schema, Spec};

use crate::config::{Arg, Config, Field, Http, RootSchema, Server, Type, Upstream};
use crate::http::Method;

trait ToDebugString: std::fmt::Debug {
    fn to_debug_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl<T: std::fmt::Debug> ToDebugString for T {}

fn map_spec_type(typ: String) -> String {
    if typ.as_str() == "Integer" {
        "Int".to_string()
    } else {
        typ
    }
}

fn get_schema_type<F: FnOnce() -> String>(get_name: F, schema: Schema) -> (bool, String) {
    let typ = schema.schema_type.unwrap().to_debug_string();
    let (is_list, type_name) = match typ.as_str() {
        "Array" => {
            if let ObjectOrReference::Ref { ref ref_path } = schema.items.unwrap().as_ref() {
                (true, ref_path.split('/').last().unwrap().to_string())
            } else {
                unreachable!()
            }
        }
        "Object" => (false, get_name()),
        _ => (false, typ),
    };

    (is_list, map_spec_type(type_name))
}

fn make_config_types(spec: &Spec) -> BTreeMap<String, Type> {
    let components = spec.components.iter().next().cloned().unwrap();
    let mut types = BTreeMap::new();
    let mut fields = BTreeMap::new();

    for (mut path, path_item) in spec.paths.clone().into_iter() {
        let (method, operation) = [
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
        .next()
        .unwrap();
        let output_type = operation
            .responses
            .get("200")
            .unwrap()
            .resolve(spec)
            .unwrap()
            .content
            .get("application/json")
            .cloned()
            .unwrap()
            .schema
            .unwrap();

        let args: BTreeMap<String, Arg> = operation
            .parameters
            .iter()
            .map(|param| {
                let param = param.resolve(spec).unwrap();
                let (is_list, name) =
                    get_schema_type(|| param.param_type.unwrap(), param.schema.unwrap());
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
            })
            .collect();

        let schema = output_type.resolve(spec).unwrap();

        let (is_list, name) = get_schema_type(
            || {
                if let ObjectOrReference::Ref { ref_path } = output_type {
                    ref_path.split('/').last().unwrap().to_string()
                } else {
                    unreachable!()
                }
            },
            schema,
        );

        if !args.is_empty() {
            let re = regex::Regex::new(r"\{\w+\}").unwrap();
            path = re
                .replacen(path.as_str(), 0, |cap: &regex::Captures| {
                    let arg_name = &cap[0][1..cap[0].len() - 1];
                    format!("{{{{args.{}}}}}", arg_name)
                })
                .to_string();
        }

        let field = Field {
            type_of: name,
            list: is_list,
            args,
            http: Some(Http { path, method, ..Default::default() }),
            ..Default::default()
        };

        fields.insert(operation.operation_id.unwrap(), field);
    }

    types.insert("Query".to_string(), Type { fields, ..Default::default() });

    for (name, component) in components.schemas.into_iter() {
        let schema = component.resolve(spec).unwrap();
        if schema.schema_type.unwrap().to_debug_string().as_str() == "Array" {
            continue;
        }

        types.insert(
            name,
            Type {
                fields: schema
                    .properties
                    .into_iter()
                    .map(|(name, property)| {
                        (
                            name.clone(),
                            Field {
                                type_of: {
                                    map_spec_type(format!(
                                        "{:?}",
                                        property.resolve(spec).unwrap().schema_type.unwrap()
                                    ))
                                },
                                required: schema.required.contains(&name),
                                ..Default::default()
                            },
                        )
                    })
                    .collect(),
                ..Default::default()
            },
        );
    }

    types
}

pub fn config_from_openapi_spec(content: &str) -> Result<Config, anyhow::Error> {
    let spec = oas3::from_reader(content.as_bytes()).unwrap();
    let types = make_config_types(&spec);
    let config = Config {
        server: Server { graphiql: Some(true), ..Default::default() },
        upstream: Upstream {
            base_url: spec.servers.first().cloned().map(|server| server.url),
            ..Default::default()
        },
        schema: RootSchema {
            query: types.get("Query").map(|_| "Query".into()),
            mutation: types.get("Mutation").map(|_| "Mutation".into()),
            ..Default::default()
        },
        types,
        unions: Default::default(),
        links: vec![],
        telemetry: Default::default(),
    };
    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;

    #[test]
    fn test_config_from_openapi_spec() {
        let spec_folder_path = Path::new("src")
            .join("config_generator")
            .join("openapi_spec");

        for spec_path in fs::read_dir(spec_folder_path).unwrap() {
            let spec_path = spec_path.unwrap().path();
            let content = fs::read_to_string(&spec_path).unwrap();
            insta::assert_snapshot!(config_from_openapi_spec(content.as_str()).unwrap().to_sdl());
        }
    }
}
