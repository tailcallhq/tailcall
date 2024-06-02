use std::collections::{BTreeMap, HashSet, VecDeque};

use convert_case::{Case, Casing};
use oas3::spec::{ObjectOrReference, SchemaType};
use oas3::{Schema, Spec};

use crate::core::config::{
    Arg, Config, Enum, Field, Http, KeyValue, RootSchema, Type, Union, Upstream,
};
use crate::core::http::Method;

fn schema_type_to_string(typ: &SchemaType) -> String {
    let typ_str = match typ {
        SchemaType::Boolean => "Boolean",
        SchemaType::Integer => "Int",
        SchemaType::Number => "Int",
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
    Enum(Enum),
}

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
            Some(ref_path.split('/').last().unwrap().to_case(Case::Pascal))
        }
        ObjectOrReference::Object(_) => None,
    }
}

#[derive(Default)]
pub struct OpenApiToConfigConverter {
    pub query: String,
    pub spec: Spec,
    pub inline_types: VecDeque<Schema>,
    pub inline_types_frozen: bool,
    pub inline_types_other: VecDeque<Schema>,
    pub unions: BTreeMap<String, Vec<Schema>>,
}

impl OpenApiToConfigConverter {
    pub fn new(query: impl AsRef<str>, spec_str: impl AsRef<str>) -> anyhow::Result<Self> {
        let spec = oas3::from_reader(spec_str.as_ref().as_bytes())?;
        Ok(Self {
            query: query.as_ref().to_string(),
            spec,
            ..Default::default()
        })
    }

    fn get_schema_type(&mut self, schema: Schema, name: Option<String>) -> TypeName {
        if let Some(element) = schema.items {
            if let Some(name) = name_from_ref_path(element.as_ref()).or_else(|| {
                let schema = element.resolve(&self.spec).ok()?;
                schema_to_primitive_type(schema.schema_type.as_ref()?)
            }) {
                TypeName::ListOf(Box::new(TypeName::Name(name)))
            } else {
                let schema = element.resolve(&self.spec).unwrap();
                TypeName::ListOf(Box::new(self.get_schema_type(schema, None)))
            }
        } else if let Some(
            typ @ (SchemaType::Integer
            | SchemaType::String
            | SchemaType::Number
            | SchemaType::Boolean),
        ) = schema.schema_type
        {
            TypeName::Name(schema_type_to_string(&typ))
        } else if schema.additional_properties.is_some() {
            TypeName::Name("JSON".to_string())
        } else if let Some(name) = name {
            TypeName::Name(name)
        } else if self.can_define_type(&schema) {
            let name = self.insert_inline_type(schema);
            TypeName::Name(name)
        } else {
            TypeName::Name("JSON".to_string())
        }
    }

    fn insert_inline_type(&mut self, schema: Schema) -> String {
        let name = format!("Type{}", self.inline_types.len());
        if self.inline_types_frozen {
            self.inline_types_other.push_back(schema);
        } else {
            self.inline_types.push_back(schema);
        }
        name
    }

    fn can_define_type(&self, schema: &Schema) -> bool {
        !schema.properties.is_empty()
            || !schema.all_of.is_empty()
            || !schema.any_of.is_empty()
            || !schema.one_of.is_empty()
            || !schema.enum_values.is_empty()
    }

    fn define_type(&mut self, schema: Schema) -> Option<UnionOrType> {
        if !schema.properties.is_empty() {
            let fields = schema
                .properties
                .into_iter()
                .map(|(name, property)| {
                    let property_schema = property.resolve(&self.spec).unwrap();
                    let (list, type_of) = self
                        .get_schema_type(property_schema.clone(), name_from_ref_path(&property))
                        .into_tuple();
                    let doc = property_schema.description.clone();
                    (
                        name.to_case(Case::Camel),
                        Field {
                            type_of,
                            required: schema.required.contains(&name),
                            list,
                            doc,
                            ..Default::default()
                        },
                    )
                })
                .collect();

            Some(UnionOrType::Type(Type {
                fields,
                doc: schema.description.clone(),
                ..Default::default()
            }))
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
                let (list, type_of) = self
                    .get_schema_type(
                        property.resolve(&self.spec).unwrap(),
                        name_from_ref_path(&property),
                    )
                    .into_tuple();
                fields.insert(
                    name.to_case(Case::Camel),
                    Field {
                        type_of,
                        list,
                        required: schema.required.contains(&name),
                        ..Default::default()
                    },
                );
            }

            Some(UnionOrType::Type(Type {
                fields,
                doc: schema.description,
                ..Default::default()
            }))
        } else if !schema.any_of.is_empty() || !schema.one_of.is_empty() {
            let types = schema
                .any_of
                .iter()
                .chain(schema.one_of.iter())
                .map(|schema| {
                    name_from_ref_path(schema)
                        .or_else(|| {
                            schema_to_primitive_type(
                                schema.resolve(&self.spec).unwrap().schema_type.as_ref()?,
                            )
                        })
                        .unwrap_or(self.insert_inline_type(schema.resolve(&self.spec).unwrap()))
                })
                .collect();

            Some(UnionOrType::Union(Union { types, doc: schema.description }))
        } else if !schema.enum_values.is_empty() {
            let variants = schema
                .enum_values
                .into_iter()
                .map(|val| format!("{val:?}"))
                .collect();
            Some(UnionOrType::Enum(Enum {
                variants,
                doc: schema.description,
            }))
        } else {
            None
        }
    }

    fn define_inline_types(
        &mut self,
        types: &mut BTreeMap<String, Type>,
        unions: &mut BTreeMap<String, Union>,
        enums: &mut BTreeMap<String, Enum>,
    ) {
        let mut index = 0;
        self.inline_types_frozen = true;
        while let Some(schema) = self.inline_types.pop_front() {
            let name = format!("Type{index}").to_case(Case::Pascal);
            match self.define_type(schema) {
                Some(UnionOrType::Type(type_)) => {
                    types.insert(name, type_);
                }
                Some(UnionOrType::Union(union)) => {
                    unions.insert(name, union);
                }
                Some(UnionOrType::Enum(enum_)) => {
                    enums.insert(name, enum_);
                }
                None => continue,
            }
            index += 1;
        }
        self.inline_types_frozen = false;
    }

    pub fn create_types_and_unions(&mut self) -> (BTreeMap<String, Type>, BTreeMap<String, Union>) {
        let components = self.spec.components.iter().next().cloned().unwrap();
        let mut types = BTreeMap::new();
        let mut fields = BTreeMap::new();
        let mut unions = BTreeMap::new();
        let mut enums = BTreeMap::new();

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
                        .get_schema_type(param.schema.clone().unwrap(), param.param_type.clone())
                        .into_tuple();
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
                .get_schema_type(
                    output_type.resolve(&self.spec).unwrap(),
                    name_from_ref_path(&output_type),
                )
                .into_tuple();

            let mut url_params = HashSet::new();
            if !args.is_empty() {
                let re = regex::Regex::new(r"\{\w+\}").unwrap();
                path = re
                    .replacen(path.as_str(), 0, |cap: &regex::Captures| {
                        let arg_name = &cap[0][1..cap[0].len() - 1];
                        url_params.insert(arg_name.to_string());
                        format!("{{{{args.{}}}}}", arg_name.to_case(Case::Camel))
                    })
                    .to_string();
            }

            let query_params = args
                .iter()
                .filter(|&(key, _)| (!url_params.contains(key)))
                .map(|(key, _)| KeyValue {
                    key: key.to_string(),
                    value: format!("{{{{args.{}}}}}", key.to_case(Case::Camel)),
                })
                .collect();

            let field = Field {
                type_of: name,
                list: is_list,
                args,
                http: Some(Http { path, method, query: query_params, ..Default::default() }),
                doc: operation.description,
                ..Default::default()
            };

            fields.insert(operation.operation_id.unwrap().to_case(Case::Camel), field);
        }

        types.insert("Query".to_string(), Type { fields, ..Default::default() });

        for (name, obj_or_ref) in components.schemas.into_iter() {
            let name = name.to_case(Case::Pascal);
            let schema = obj_or_ref.resolve(&self.spec).unwrap();
            match self.define_type(schema) {
                Some(UnionOrType::Type(type_)) => {
                    types.insert(name, type_);
                }
                Some(UnionOrType::Union(union)) => {
                    unions.insert(name, union);
                }
                Some(UnionOrType::Enum(enum_)) => {
                    enums.insert(name, enum_);
                }
                None => continue,
            }
        }

        self.define_inline_types(&mut types, &mut unions, &mut enums);

        (types, unions)
    }

    pub fn convert(&mut self) -> Config {
        let (types, unions) = self.create_types_and_unions();
        let config = Config {
            server: Default::default(),
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
            .join("core")
            .join("generator")
            .join("openapi");

        for spec_path in fs::read_dir(spec_folder_path).unwrap() {
            let spec_path = spec_path.unwrap().path();
            let content = fs::read_to_string(&spec_path).unwrap();
            insta::assert_snapshot!(OpenApiToConfigConverter::new("Query", content.as_str())
                .unwrap()
                .convert()
                .to_sdl());
        }
    }
}
