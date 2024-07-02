use async_graphql_parser::types::{
    EnumType, EnumValueDefinition, TypeDefinition, TypeKind, TypeSystemDefinition,
};
use async_graphql_value::Name;

use crate::common::pos;

pub fn into_enum_definition(enum_values: Option<Vec<String>>, name: &str) -> TypeSystemDefinition {
    let mut enum_value_defintions = vec![];
    if let Some(enum_values) = enum_values {
        for enum_value in enum_values {
            let formated_value: String = enum_value
                .to_string()
                .chars()
                .filter(|ch| ch != &'"')
                .collect();
            enum_value_defintions.push(pos(EnumValueDefinition {
                value: pos(Name::new(formated_value)),
                description: None,
                directives: vec![],
            }));
        }
    }

    TypeSystemDefinition::Type(pos(TypeDefinition {
        name: pos(Name::new(name)),
        kind: TypeKind::Enum(EnumType { values: enum_value_defintions }),
        description: None,
        directives: vec![],
        extend: false,
    }))
}
