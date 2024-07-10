use async_graphql::parser::types::{
    EnumType, EnumValueDefinition, TypeDefinition, TypeKind, TypeSystemDefinition,
};
use async_graphql::{Name, Positioned};
use schemars::schema::Schema;

#[derive(Debug)]
pub struct EnumValue {
    pub variants: Vec<String>,
    pub description: Option<Positioned<String>>,
}

use crate::common::{get_description, pos};

pub fn into_enum_definition(enum_value: EnumValue, name: &str) -> TypeSystemDefinition {
    let mut enum_value_definition = vec![];
    for enum_value in enum_value.variants {
        let formatted_value: String = enum_value
            .to_string()
            .chars()
            .filter(|ch| ch != &'"')
            .collect();
        enum_value_definition.push(pos(EnumValueDefinition {
            value: pos(Name::new(formatted_value)),
            description: None,
            directives: vec![],
        }));
    }

    TypeSystemDefinition::Type(pos(TypeDefinition {
        name: pos(Name::new(name)),
        kind: TypeKind::Enum(EnumType { values: enum_value_definition }),
        description: enum_value.description,
        directives: vec![],
        extend: false,
    }))
}

pub fn into_enum_value(obj: &Schema) -> Option<EnumValue> {
    match obj {
        Schema::Object(schema_object) => {
            let description = get_description(schema_object);
            if let Some(enum_values) = &schema_object.enum_values {
                return Some(EnumValue {
                    variants: enum_values
                        .iter()
                        .map(|val| val.to_string())
                        .collect::<Vec<String>>(),
                    description: description.map(|description| pos(description.to_owned())),
                });
            }
            None
        }
        _ => None,
    }
}
