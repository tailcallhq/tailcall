use std::collections::HashMap;

use convert_case::{Case, Casing};
use oas3::spec::{ObjectOrReference, SchemaType};
use oas3::{OpenApiV3Spec, Schema};

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

pub fn unknown_type(types: &mut HashMap<String, Schema>, schema: Schema) -> String {
    let name = format!("Type{}", types.len());
    types.insert(name.clone(), schema);
    name
}

pub fn get_schema_type(
    spec: &OpenApiV3Spec,
    schema: Schema,
    name: Option<String>,
    types: &mut HashMap<String, Schema>,
) -> anyhow::Result<TypeName> {
    Ok(if let Some(element) = schema.items {
        let inner_schema = element.resolve(spec)?;
        if inner_schema.schema_type == Some(SchemaType::String)
            && !inner_schema.enum_values.is_empty()
        {
            TypeName::ListOf(Box::new(TypeName::Name(unknown_type(types, inner_schema))))
        } else if let Some(name) = name_from_ref_path(element.as_ref())
            .or_else(|| schema_to_primitive_type(inner_schema.schema_type.as_ref()?))
        {
            TypeName::ListOf(Box::new(TypeName::Name(name)))
        } else {
            TypeName::ListOf(Box::new(get_schema_type(spec, inner_schema, None, types)?))
        }
    } else if schema.schema_type == Some(SchemaType::String) && !schema.enum_values.is_empty() {
        TypeName::Name(unknown_type(types, schema))
    } else if let Some(
        typ @ (SchemaType::Integer | SchemaType::String | SchemaType::Number | SchemaType::Boolean),
    ) = schema.schema_type
    {
        TypeName::Name(schema_type_to_string(&typ))
    } else if let Some(name) = name {
        TypeName::Name(name)
    } else if can_define_type(&schema) {
        TypeName::Name(unknown_type(types, schema))
    } else {
        TypeName::Name("JSON".to_string())
    })
}
