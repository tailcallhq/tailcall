use std::collections::HashSet;

use convert_case::{Case, Casing};
use oas3::spec::{ObjectOrReference, SchemaType};
use oas3::{OpenApiV3Spec, Schema};

use crate::core::config::{Config, Enum, Field, Type, Union, Variant};
use crate::core::generator::openapi::anonymous_type_generator::AnonymousTypes;
use crate::core::valid::{Valid, Validator};

pub const TYPE_FIELD: &str = "type_";

///
/// The TypeName enum represents the name of a type in the generated code.
/// Creating a special type is required since the types can be recursive
#[derive(Debug)]
pub enum TypeName {
    ListOf(Box<TypeName>),
    Name(String),
}

impl TypeName {
    pub fn name(&self) -> Option<String> {
        match self {
            TypeName::ListOf(_) => None,
            TypeName::Name(name) => Some(name.clone()),
        }
    }

    pub fn into_tuple(self) -> (bool, String) {
        match self {
            TypeName::ListOf(inner) => (true, inner.name().unwrap()),
            TypeName::Name(name) => (false, name),
        }
    }
}

pub fn name_from_ref_path<T>(obj_or_ref: &ObjectOrReference<T>) -> Option<String> {
    match obj_or_ref {
        ObjectOrReference::Ref { ref_path } => {
            ref_path.split('/').last().map(|a| a.to_case(Case::Pascal))
        }
        ObjectOrReference::Object(_) => None,
    }
}

pub fn schema_type_to_string(typ: &SchemaType) -> String {
    match typ {
        x @ (SchemaType::Boolean | SchemaType::String | SchemaType::Array | SchemaType::Object) => {
            format!("{x:?}")
        }
        SchemaType::Integer | SchemaType::Number => "Int".into(),
    }
}

pub fn schema_to_primitive_type(typ: &SchemaType) -> Option<String> {
    match typ {
        SchemaType::Array | SchemaType::Object => None,
        x => Some(schema_type_to_string(x)),
    }
}

pub fn can_define_type(schema: &Schema) -> bool {
    !schema.properties.is_empty()
        || !schema.all_of.is_empty()
        || !schema.any_of.is_empty()
        || !schema.one_of.is_empty()
        || !schema.enum_values.is_empty()
}

pub fn get_all_of_properties(
    spec: &OpenApiV3Spec,
    properties: &mut Vec<(String, ObjectOrReference<Schema>)>,
    required: &mut HashSet<String>,
    schema: Schema,
) {
    required.extend(schema.required);
    if !schema.all_of.is_empty() {
        for obj in schema.all_of {
            let schema = obj.resolve(spec).unwrap();
            get_all_of_properties(spec, properties, required, schema);
        }
    }
    properties.extend(schema.properties);
}

pub fn get_schema_type(
    spec: &OpenApiV3Spec,
    schema: Schema,
    name: Option<String>,
    types: &mut AnonymousTypes,
) -> anyhow::Result<TypeName> {
    Ok(if let Some(element) = schema.items {
        let inner_schema = element.resolve(spec)?;
        if inner_schema.schema_type == Some(SchemaType::String)
            && !inner_schema.enum_values.is_empty()
        {
            TypeName::ListOf(Box::new(TypeName::Name(types.add(inner_schema))))
        } else if let Some(name) = name_from_ref_path(element.as_ref())
            .or_else(|| schema_to_primitive_type(inner_schema.schema_type.as_ref()?))
        {
            TypeName::ListOf(Box::new(TypeName::Name(name)))
        } else {
            TypeName::ListOf(Box::new(get_schema_type(spec, inner_schema, None, types)?))
        }
    } else if schema.schema_type == Some(SchemaType::String) && !schema.enum_values.is_empty() {
        TypeName::Name(types.add(schema))
    } else if let Some(
        typ @ (SchemaType::Integer | SchemaType::String | SchemaType::Number | SchemaType::Boolean),
    ) = schema.schema_type
    {
        TypeName::Name(schema_type_to_string(&typ))
    } else if let Some(name) = name {
        TypeName::Name(name)
    } else if can_define_type(&schema) {
        TypeName::Name(types.add(schema))
    } else {
        TypeName::Name("JSON".to_string())
    })
}

#[allow(clippy::too_many_arguments)]
pub fn define_type(
    spec: &OpenApiV3Spec,
    config: &mut Config,
    name: String,
    schema: Schema,
    types: &mut AnonymousTypes,
) -> Valid<(), String> {
    if !schema.properties.is_empty() {
        Valid::from_iter(schema.properties, |(name, property)| {
            let property_schema = match property.resolve(spec) {
                Ok(schema) => schema,
                Err(err) => return Valid::fail(err.to_string()),
            };

            let type_name = get_schema_type(
                spec,
                property_schema.clone(),
                name_from_ref_path(&property),
                types,
            );
            let (list, type_of) = match type_name {
                Ok(type_name) => type_name.into_tuple(),
                Err(err) => return Valid::fail(err.to_string()),
            };

            let doc = property_schema.description.clone();
            Valid::succeed((
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
        .map(|fields| {
            config.types.insert(
                name,
                Type {
                    fields: fields.into_iter().collect(),
                    doc: schema.description.clone(),
                    ..Default::default()
                },
            );
        })
    } else if !schema.all_of.is_empty() {
        let mut properties: Vec<_> = vec![];
        let mut required = HashSet::new();
        let doc = schema.description.clone();
        get_all_of_properties(spec, &mut properties, &mut required, schema);

        Valid::from_iter(properties, |(name, property)| {
            let type_name = get_schema_type(
                spec,
                property.resolve(spec).unwrap(),
                name_from_ref_path(&property),
                types,
            );

            let (list, type_of) = match type_name {
                Ok(val) => val.into_tuple(),
                Err(err) => return Valid::fail(err.to_string()),
            };

            Valid::succeed((
                name.clone(),
                Field {
                    type_of,
                    list,
                    required: required.contains(&name),
                    ..Default::default()
                },
            ))
        })
        .map(|fields| {
            let fields = fields.into_iter().collect();
            config
                .types
                .insert(name, Type { fields, doc, ..Default::default() });
        })
    } else if !schema.any_of.is_empty() || !schema.one_of.is_empty() {
        Valid::from_iter(schema.any_of.iter().chain(schema.one_of.iter()), |schema| {
            let type_name = match name_from_ref_path(schema) {
                Some(type_name) => Some(type_name),
                None => {
                    let schema = match schema.resolve(spec) {
                        Ok(schema) => schema,
                        Err(err) => return Valid::fail(err.to_string()),
                    };
                    schema
                        .schema_type
                        .as_ref()
                        .and_then(schema_to_primitive_type)
                }
            };

            if let Some(type_name) = type_name {
                return Valid::succeed(type_name);
            }

            match schema.resolve(spec) {
                Ok(schema) => Valid::succeed(types.add(schema)),
                Err(err) => Valid::fail(err.to_string()),
            }
        })
        .map(|types| {
            config.unions.insert(
                name,
                Union { types: types.into_iter().collect(), doc: schema.description },
            );
        })
    } else if !schema.enum_values.is_empty() {
        Valid::from_iter(schema.enum_values, |val| match val {
            serde_yaml::Value::String(string) => Valid::succeed(string),
            _ => Valid::fail("Enum values must be strings".to_string()),
        })
        .map(|variants| {
            let variants = variants
                .into_iter()
                .map(|name| Variant { name, alias: None })
                .collect();
            config
                .enums
                .insert(name, Enum { variants, doc: schema.description });
        })
    } else {
        return Valid::fail("Unable to define type".to_string());
    }
}
