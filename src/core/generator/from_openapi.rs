use oas3::{OpenApiV3Spec, Spec};

use crate::core::config::Config;
use crate::core::generator::json;
use crate::core::generator::openapi::QueryGenerator;
use crate::core::transform::{Transform, TransformerOps};
use crate::core::valid::{Valid, Validator};

#[derive(Default)]
pub struct FromOpenAPIGenerator {
    query: String,
    #[allow(unused)]
    spec: Spec,
}

impl FromOpenAPIGenerator {
    pub fn new(query: String, spec: OpenApiV3Spec) -> Self {
        Self { query, spec }
    }
}

impl Transform for FromOpenAPIGenerator {
    type Value = Config;
    type Error = String;

    fn transform(&self, value: Self::Value) -> Valid<Self::Value, Self::Error> {
        json::SchemaGenerator::new(self.query.clone())
            .pipe(QueryGenerator::new(self.query.as_str(), &self.spec))
            .transform(value)
    }
}

pub fn from_openapi_spec(query: &str, spec: OpenApiV3Spec) -> Config {
    let config = Config::default();
    let final_config = FromOpenAPIGenerator::new(query.to_string(), spec)
        .transform(config)
        .to_result();
    final_config.unwrap_or_else(|e| {
        tracing::warn!("Failed to generate config from OpenAPI spec: {}", e);
        Config::default()
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_openapi_apis_guru() {
        let apis_guru = config_from_openapi_spec("apis-guru.yml");
        insta::assert_snapshot!(apis_guru);
    }

    #[test]
    fn test_openapi_jsonplaceholder() {
        let jsonplaceholder = config_from_openapi_spec("jsonplaceholder.yml");
        insta::assert_snapshot!(jsonplaceholder);
    }

    #[test]
    fn test_openapi_spotify() {
        let spotify = config_from_openapi_spec("spotify.yml");
        insta::assert_snapshot!(spotify);
    }

    fn config_from_openapi_spec(filename: &str) -> String {
        let spec_path = Path::new("src")
            .join("core")
            .join("generator")
            .join("tests")
            .join("fixtures")
            .join("openapi")
            .join(filename);

        let spec = oas3::from_path(spec_path).unwrap();
        from_openapi_spec("Query", spec).to_sdl()
    }
}

/*
use std::collections::{BTreeMap, BTreeSet, HashSet};

use convert_case::{Case, Casing};
use oas3::spec::{ObjectOrReference, SchemaType};
use oas3::{OpenApiV3Spec, Schema, Spec};

use crate::core::config::{Config, Enum, Field, Http, Type, Union, Variant};
use crate::core::http::Method;

#[derive(Default)]
pub struct OpenApiToConfigConverter {
    #[allow(unused)]
    spec: Spec,
    config: Config,
    anonymous_types: BTreeMap<String, Schema>,
}

///
/// The TypeName enum represents the name of a type in the generated code.
/// Creating a special type is required since the types can be recursive
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

fn schema_type_to_string(typ: &SchemaType) -> String {
    match typ {
        x @ (SchemaType::Boolean | SchemaType::String | SchemaType::Array | SchemaType::Object) => {
            format!("{x:?}")
        }
        SchemaType::Integer | SchemaType::Number => "Int".into(),
    }
}

fn schema_to_primitive_type(typ: &SchemaType) -> Option<String> {
    match typ {
        SchemaType::Array | SchemaType::Object => None,
        x => Some(schema_type_to_string(x)),
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

impl OpenApiToConfigConverter {
    pub fn new(spec: OpenApiV3Spec) -> anyhow::Result<Self> {
        let config = Config::default();
        Ok(Self { config, spec, anonymous_types: Default::default() })
    }

    pub fn define_queries(mut self) -> Self {
        self.config = self.config.query("Query");

        let fields: BTreeMap<String, Field> = self
            .spec
            .paths
            .clone()
            .into_iter()
            .filter_map(|(path, path_item)| {
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
                .next()?;

                let Ok(response) = operation
                    .responses
                    .first_key_value()
                    .map(|(_, v)| v)?
                    .resolve(&self.spec)
                else {
                    tracing::warn!("skipping {path}: no sample response found");
                    None?
                };

                let Some(output_type) = response
                    .content
                    .first_key_value()
                    .map(|(_, v)| v)
                    .cloned()
                    .and_then(|v| v.schema)
                else {
                    tracing::warn!("skipping {path}: unable to detect output type");
                    None?
                };

                match name_from_ref_path(&output_type) {
                    Some(type_of) => {
                        let field = Field {
                            type_of,
                            http: Some(Http { path, method, ..Default::default() }),
                            doc: operation.description,
                            ..Default::default()
                        };

                        Some((operation.operation_id?.to_case(Case::Camel), field))
                    }
                    None => {
                        tracing::warn!("skipping {path}: unable to find name of the type");
                        None
                    }
                }
            })
            .collect();

        if let Some(query) = self.config.schema.query.as_ref() {
            self.config
                .types
                .insert(query.to_string(), Type { fields, ..Default::default() });
        }

        self
    }

    fn insert_anonymous_type(&mut self, schema: Schema) -> String {
        let name = format!("Type{}", self.anonymous_types.len());
        self.anonymous_types.insert(name.clone(), schema);
        name
    }

    fn can_define_type(&self, schema: &Schema) -> bool {
        !schema.properties.is_empty()
            || !schema.all_of.is_empty()
            || !schema.any_of.is_empty()
            || !schema.one_of.is_empty()
            || !schema.enum_values.is_empty()
    }

    fn get_schema_type(
        &mut self,
        schema: Schema,
        name: Option<String>,
    ) -> anyhow::Result<TypeName> {
        Ok(if let Some(element) = schema.items {
            let inner_schema = element.resolve(&self.spec)?;
            if inner_schema.schema_type == Some(SchemaType::String)
                && !inner_schema.enum_values.is_empty()
            {
                let name = self.insert_anonymous_type(inner_schema);
                TypeName::ListOf(Box::new(TypeName::Name(name)))
            } else if let Some(name) = name_from_ref_path(element.as_ref())
                .or_else(|| schema_to_primitive_type(inner_schema.schema_type.as_ref()?))
            {
                TypeName::ListOf(Box::new(TypeName::Name(name)))
            } else {
                TypeName::ListOf(Box::new(self.get_schema_type(inner_schema, None)?))
            }
        } else if schema.schema_type == Some(SchemaType::String) && !schema.enum_values.is_empty() {
            let name = self.insert_anonymous_type(schema);
            TypeName::Name(name)
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
            let name = self.insert_anonymous_type(schema);
            TypeName::Name(name)
        } else {
            TypeName::Name("JSON".to_string())
        })
    }

    fn get_all_of_properties(
        &self,
        properties: &mut Vec<(String, ObjectOrReference<Schema>)>,
        required: &mut HashSet<String>,
        schema: Schema,
    ) {
        required.extend(schema.required);
        if !schema.all_of.is_empty() {
            for obj in schema.all_of {
                let schema = obj.resolve(&self.spec).unwrap();
                self.get_all_of_properties(properties, required, schema);
            }
        }
        properties.extend(schema.properties);
    }

    fn define_type(&mut self, name: String, schema: Schema) -> anyhow::Result<()> {
        if !schema.properties.is_empty() {
            let fields = schema
                .properties
                .into_iter()
                .map(|(name, property)| {
                    let property_schema = property.resolve(&self.spec)?;
                    let (list, type_of) = self
                        .get_schema_type(property_schema.clone(), name_from_ref_path(&property))?
                        .into_tuple();
                    let doc = property_schema.description.clone();
                    Ok((
                        name.clone(),
                        Field {
                            type_of,
                            required: schema.required.contains(&name),
                            list,
                            doc,
                            ..Default::default()
                        },
                    ))
                })
                .collect::<anyhow::Result<BTreeMap<String, Field>>>()?;

            self.config.types.insert(
                name,
                Type {
                    fields,
                    doc: schema.description.clone(),
                    ..Default::default()
                },
            );
        } else if !schema.all_of.is_empty() {
            let mut properties: Vec<_> = vec![];
            let mut required = HashSet::new();
            let doc = schema.description.clone();
            self.get_all_of_properties(&mut properties, &mut required, schema);

            let mut fields = BTreeMap::new();

            for (name, property) in properties.into_iter() {
                let (list, type_of) = self
                    .get_schema_type(property.resolve(&self.spec)?, name_from_ref_path(&property))?
                    .into_tuple();
                fields.insert(
                    name.clone(),
                    Field {
                        type_of,
                        list,
                        required: required.contains(&name),
                        ..Default::default()
                    },
                );
            }

            self.config
                .types
                .insert(name, Type { fields, doc, ..Default::default() });
        } else if !schema.any_of.is_empty() || !schema.one_of.is_empty() {
            let types = schema
                .any_of
                .iter()
                .chain(schema.one_of.iter())
                .map(|schema| {
                    // try getting the name of the type
                    let name = name_from_ref_path(schema);

                    match name {
                        Some(name) => Ok(name),
                        None => {
                            let resolved_schema = schema.resolve(&self.spec)?;
                            // check if the schema is a primitive type
                            let name = resolved_schema
                                .schema_type
                                .as_ref()
                                .and_then(schema_to_primitive_type)
                                .unwrap_or(self.insert_anonymous_type(resolved_schema));

                            Ok(name)
                        }
                    }
                })
                .collect::<anyhow::Result<BTreeSet<String>>>()?;

            self.config
                .unions
                .insert(name, Union { types, doc: schema.description });
        } else if !schema.enum_values.is_empty() {
            let variants = schema
                .enum_values
                .into_iter()
                .map(|val| match val {
                    serde_yaml::Value::String(string) => Variant { name: string, alias: None },
                    _ => unreachable!(),
                })
                .collect();
            self.config
                .enums
                .insert(name, Enum { variants, doc: schema.description });
        } else {
            anyhow::bail!("Unknown schema type");
        }

        Ok(())
    }

    fn define_types(mut self) -> Self {
        if let Some(components) = self.spec.components.clone() {
            for (name, obj_or_ref) in components.schemas.into_iter() {
                let name = name.to_case(Case::Pascal);
                let schema = obj_or_ref
                    .resolve(&self.spec)
                    .map_err(|err| anyhow::anyhow!("{err}"));
                if let Err(err) = schema.and_then(|schema| self.define_type(name.clone(), schema)) {
                    tracing::warn!("skipping {name}: {err}");
                }
            }
        }

        self
    }

    pub fn convert(mut self) -> Config {
        self = self.define_queries();
        self = self.define_types();
        self.config
    }
}

pub fn from_openapi_spec(spec: OpenApiV3Spec) -> anyhow::Result<Config> {
    OpenApiToConfigConverter::new(spec).map(|converter| converter.convert())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_openapi_apis_guru() {
        let apis_guru = config_from_openapi_spec("apis-guru.yml").unwrap();
        insta::assert_snapshot!(apis_guru);
    }

    #[test]
    fn test_openapi_jsonplaceholder() {
        let jsonplaceholder = config_from_openapi_spec("jsonplaceholder.yml").unwrap();
        insta::assert_snapshot!(jsonplaceholder);
    }

    #[test]
    fn test_openapi_spotify() {
        let spotify = config_from_openapi_spec("spotify.yml").unwrap();
        insta::assert_snapshot!(spotify);
    }

    fn config_from_openapi_spec(filename: &str) -> Option<String> {
        let spec_path = Path::new("src")
            .join("core")
            .join("generator")
            .join("tests")
            .join("fixtures")
            .join("openapi")
            .join(filename);

        let spec = oas3::from_path(spec_path).unwrap();
        from_openapi_spec(spec).ok().map(|config| config.to_sdl())
    }
}

 */