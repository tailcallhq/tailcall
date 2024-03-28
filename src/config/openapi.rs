use std::collections::BTreeMap;
use oas3::spec::{ObjectOrReference};
use oas3::Spec;
use crate::config::{Config, Field, Http, RootSchema, Type, Upstream};

fn map_spec_type(typ: String) -> String {
    if typ.as_str() == "Integer" {
        "Int".to_string()
    } else {
        typ
    }
}

fn make_config_types(spec: &Spec) -> BTreeMap<String, Type> {
    let components = spec.components.iter().next().cloned().unwrap();
    let mut types = BTreeMap::new();
    let mut fields = BTreeMap::new();

    for (path, path_item) in spec.paths.clone().into_iter() {
        let operation = [
            path_item.get,
            path_item.put,
            path_item.post,
            path_item.delete,
            path_item.options,
            path_item.head,
            path_item.patch,
            path_item.trace,
        ]
        .into_iter()
        .flatten()
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

        let name = if let ObjectOrReference::Ref { ref ref_path } = output_type {
            ref_path.split('/').last().unwrap().to_string()
        } else {
            unreachable!()
        };

        fields.insert(
            operation.operation_id.unwrap(),
            Field {
                type_of: name,
                http: Some(Http { path, ..Default::default() }),
                ..Default::default()
            },
        );
    }

    types.insert("Query".to_string(), Type { fields, ..Default::default() });

    for (name, component) in components.schemas.into_iter() {
        let schema = component.resolve(spec).unwrap();

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
                                    
                                    // if schema.required.contains(&name) {
                                    //     typ.push('!');
                                    // }
                                    map_spec_type(format!(
                                        "{:?}",
                                        property.resolve(spec).unwrap().schema_type.unwrap()
                                    ))
                                },
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
    let config = Config {
        server: Default::default(),
        upstream: Upstream {
            base_url: spec.servers.first().cloned().map(|server| server.url),
            ..Default::default()
        },
        schema: RootSchema { query: Some("Query".into()), ..Default::default() },
        types: make_config_types(&spec),
        unions: Default::default(),
        links: vec![],
        telemetry: Default::default(),
    };

    println!("{}", config.to_sdl());
    Ok(config)
}
