use std::collections::{BTreeMap, HashSet};

use async_graphql_parser::types::{DirectiveLocation, TypeSystemDefinition};
use async_graphql_value::Name;
use schemars::schema::{RootSchema, Schema, SchemaObject};
use schemars::JsonSchema;

use crate::common::{
    extract_enum_values, first_char_to_lower, first_char_to_upper, get_description, pos,
};
use crate::enum_definition::into_enum_definition;
use crate::input_definition::{into_input_definition, into_input_value_definition};

pub trait DirectiveDefinition {
    fn into_schemars() -> RootSchema
    where
        Self: JsonSchema,
    {
        schemars::schema_for!(Self)
    }

    fn directive_definition(generated_types: &mut HashSet<String>) -> Vec<TypeSystemDefinition>;
}

#[derive(Clone)]
pub struct Attrs {
    pub name: &'static str,
    pub repeatable: bool,
    pub locations: Vec<&'static str>,
    pub is_lowercase_name: bool,
}

pub fn from_directive_location(str: DirectiveLocation) -> String {
    match str {
        DirectiveLocation::Schema => String::from("SCHEMA"),
        DirectiveLocation::Object => String::from("OBJECT"),
        DirectiveLocation::FieldDefinition => String::from("FIELD_DEFINITION"),
        DirectiveLocation::EnumValue => String::from("ENUM_VALUE_DEFINITION"),
        _ => String::from("FIELD_DEFINITION"),
    }
}

fn into_directive_location(str: &str) -> DirectiveLocation {
    match str {
        "Schema" => DirectiveLocation::Schema,
        "Object" => DirectiveLocation::Object,
        "FieldDefinition" => DirectiveLocation::FieldDefinition,
        "EnumValue" => DirectiveLocation::EnumValue,
        _ => DirectiveLocation::FieldDefinition,
    }
}

pub fn into_directive_definition(
    root_schema: RootSchema,
    attrs: Attrs,
    generated_types: &mut HashSet<String>,
) -> Vec<TypeSystemDefinition> {
    let mut service_doc_definitions = vec![];
    let definitions: BTreeMap<String, Schema> = root_schema.definitions;
    let schema: SchemaObject = root_schema.schema;
    let description = get_description(&schema);

    for (mut name, schema) in definitions.into_iter() {
        if generated_types.contains(&name) {
            continue;
        }
        let enum_values = extract_enum_values(&schema);
        if enum_values.is_some() {
            service_doc_definitions.push(into_enum_definition(enum_values, &name));
            generated_types.insert(name.to_string());
        } else {
            generated_types.insert(name.to_string());
            first_char_to_upper(&mut name);
            service_doc_definitions.push(into_input_definition(
                schema.clone().into_object(),
                name.as_str(),
            ));
        }
    }

    let name = if attrs.is_lowercase_name {
        attrs.name.to_lowercase()
    } else {
        first_char_to_lower(attrs.name)
    };

    let directve_definition =
        TypeSystemDefinition::Directive(pos(async_graphql_parser::types::DirectiveDefinition {
            description: description.map(|inner| pos(inner.clone())),
            name: pos(Name::new(name)),
            arguments: into_input_value_definition(&schema),
            is_repeatable: attrs.repeatable,
            locations: attrs
                .locations
                .into_iter()
                .map(|val| pos(into_directive_location(val)))
                .collect(),
        }));
    service_doc_definitions.push(directve_definition);
    service_doc_definitions
}
