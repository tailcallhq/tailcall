use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use oas3::spec::{ObjectOrReference, SchemaType};
use oas3::{Schema, Spec};

use crate::config::{Arg, Config, Field, Http, RootSchema, Server, Type, Union, Upstream};
use crate::http::Method;

fn schema_type_to_string(typ: &SchemaType) -> String {
    let typ_str = match typ {
        SchemaType::Boolean => "Boolean",
        SchemaType::Integer => "Int",
        SchemaType::Number => "Number",
        SchemaType::String => "String",
        SchemaType::Array => "Array",
        SchemaType::Object => "Object",
    };

    typ_str.to_string()
}

fn schema_to_primitive_type(typ: &SchemaType) -> Option<String> {
    match typ {
        SchemaType::Array | SchemaType::Object => None,
        x => Some(schema_type_to_string(x)),
    }
}

enum UnionOrType {
    Union(Union),
    Type(Type),
}

fn name_from_ref_path<T>(obj_or_ref: &ObjectOrReference<T>) -> Option<String> {
    match obj_or_ref {
        ObjectOrReference::Ref { ref_path } => {
            Some(ref_path.split('/').last().unwrap().to_string())
        }
        ObjectOrReference::Object(_) => None,
    }
}

#[derive(Default)]
pub struct OpenApiToGraphQLConverter {
    pub spec: Spec,
    pub inline_types: BTreeMap<String, Schema>,
    pub unions: BTreeMap<String, Vec<Schema>>,
}

impl OpenApiToGraphQLConverter {
    pub fn new(spec_str: impl AsRef<str>) -> anyhow::Result<Self> {
        let spec = oas3::from_reader(spec_str.as_ref().as_bytes())?;
        Ok(Self { spec, ..Default::default() })
    }

    fn get_schema_type<F: Fn() -> Option<String>>(
        &mut self,
        schema: Schema,
        get_name: F,
    ) -> (bool, String) {
        if let Some(element) = schema.items {
            if let Some(name) = name_from_ref_path(element.as_ref()).or_else(|| {
                let schema = element.resolve(&self.spec).ok()?;
                schema_to_primitive_type(schema.schema_type.as_ref().unwrap())
            }) {
                (true, name)
            } else {
                (
                    true,
                    self.insert_inline_type(element.resolve(&self.spec).unwrap()),
                )
            }
        } else if let Some(
            typ @ (SchemaType::Integer
            | SchemaType::String
            | SchemaType::Number
            | SchemaType::Boolean),
        ) = schema.schema_type
        {
            (false, schema_type_to_string(&typ))
        } else if let Some(name) = get_name() {
            (false, name)
        } else {
            let name = self.insert_inline_type(schema);
            (false, name)
        }
    }

    fn insert_inline_type(&mut self, schema: Schema) -> String {
        let name = format!("Type{}", self.inline_types.len());
        self.inline_types.insert(name.clone(), schema);
        name
    }

    fn define_type(&mut self, schema: Schema) -> UnionOrType {
        if !schema.properties.is_empty() {
            let fields = schema
                .properties
                .into_iter()
                .map(|(name, property)| {
                    let property_schema = property.resolve(&self.spec).unwrap();
                    let doc = property_schema.description.clone();
                    (
                        name.to_case(Case::Camel),
                        Field {
                            type_of: {
                                if let Some(typ) = property_schema.schema_type.as_ref() {
                                    schema_type_to_string(typ)
                                } else {
                                    self.insert_inline_type(property_schema)
                                }
                            },
                            required: schema.required.contains(&name),
                            doc,
                            ..Default::default()
                        },
                    )
                })
                .collect();

            UnionOrType::Type(Type {
                fields,
                doc: schema.description.clone(),
                ..Default::default()
            })
        } else if !schema.all_of.is_empty() {
            let properties: Vec<_> = schema
                .all_of
                .into_iter()
                .flat_map(|obj_or_ref| {
                    obj_or_ref
                        .resolve(&self.spec)
                        .unwrap()
                        .properties
                        .into_iter()
                })
                .collect();

            let mut fields = BTreeMap::new();

            for (name, property) in properties.into_iter() {
                fields.insert(
                    name.to_case(Case::Camel),
                    Field {
                        type_of: {
                            let schema = property.resolve(&self.spec).unwrap();
                            if let Some(typ) = schema.schema_type.as_ref() {
                                schema_type_to_string(typ)
                            } else {
                                self.insert_inline_type(schema)
                            }
                        },
                        required: schema.required.contains(&name),
                        ..Default::default()
                    },
                );
            }

            UnionOrType::Type(Type { fields, doc: schema.description, ..Default::default() })
        } else if !schema.any_of.is_empty() {
            let types = schema
                .any_of
                .iter()
                .map(|schema| name_from_ref_path(schema).unwrap_or("AnyOf".into()))
                .collect();

            UnionOrType::Union(Union { types, doc: schema.description })
        } else if !schema.enum_values.is_empty() {
            let variants = schema
                .enum_values
                .into_iter()
                .map(|val| format!("{val:?}"))
                .collect();
            UnionOrType::Type(Type {
                variants: Some(variants),
                doc: schema.description,
                ..Default::default()
            })
        } else {
            UnionOrType::Type(Type::default())
        }
    }

    fn define_inline_types(
        &mut self,
        types: &mut BTreeMap<String, Type>,
        unions: &mut BTreeMap<String, Union>,
    ) {
        while let Some((name, schema)) = self.inline_types.pop_last() {
            let name = name.to_case(Case::Pascal);
            match self.define_type(schema) {
                UnionOrType::Type(type_) => {
                    types.insert(name, type_);
                }
                UnionOrType::Union(union) => {
                    unions.insert(name, union);
                }
            }
        }
    }

    pub fn create_types_and_unions(&mut self) -> (BTreeMap<String, Type>, BTreeMap<String, Union>) {
        let components = self.spec.components.iter().next().cloned().unwrap();
        let mut types = BTreeMap::new();
        let mut fields = BTreeMap::new();
        let mut unions = BTreeMap::new();

        for (mut path, path_item) in self.spec.paths.clone().into_iter() {
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

            let Ok(response) = operation
                .responses
                .first_key_value()
                .map(|(_, v)| v)
                .unwrap()
                .resolve(&self.spec)
            else {
                continue;
            };

            let Some(output_type) = response
                .content
                .first_key_value()
                .map(|(_, v)| v)
                .cloned()
                .and_then(|v| v.schema)
            else {
                continue;
            };

            let args: BTreeMap<String, Arg> = operation
                .parameters
                .iter()
                .map(|param| {
                    let param = param.resolve(&self.spec).unwrap();

                    let (is_list, name) = self
                        .get_schema_type(param.schema.clone().unwrap(), || {
                            param.param_type.clone()
                        });
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
                })
                .collect();

            let (is_list, name) = self
                .get_schema_type(output_type.resolve(&self.spec).unwrap(), || {
                    name_from_ref_path(&output_type)
                });

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
                doc: operation.description,
                ..Default::default()
            };

            fields.insert(operation.operation_id.unwrap().to_case(Case::Camel), field);
        }

        types.insert("Query".to_string(), Type { fields, ..Default::default() });

        for (name, obj_or_ref) in components.schemas.into_iter() {
            let name = name.to_case(Case::Pascal);
            let schema = obj_or_ref.resolve(&self.spec).unwrap();
            if let Some(SchemaType::Array) = schema.schema_type {
                continue;
            }
            match self.define_type(schema) {
                UnionOrType::Type(type_) => {
                    types.insert(name, type_);
                }
                UnionOrType::Union(union) => {
                    unions.insert(name, union);
                }
            }
        }

        self.define_inline_types(&mut types, &mut unions);

        (types, unions)
    }

    pub fn convert(&mut self) -> Config {
        let (types, unions) = self.create_types_and_unions();
        let config = Config {
            server: Server { graphiql: Some(true), ..Default::default() },
            upstream: Upstream {
                base_url: self.spec.servers.first().cloned().map(|server| server.url),
                ..Default::default()
            },
            schema: RootSchema {
                query: types.get("Query").map(|_| "Query".into()),
                ..Default::default()
            },
            types,
            unions,
            ..Default::default()
        };
        config
    }
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
            insta::assert_snapshot!(OpenApiToGraphQLConverter::new(content.as_str())
                .unwrap()
                .convert()
                .to_sdl());
        }
    }
}
