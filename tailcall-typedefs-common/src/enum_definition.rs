use async_graphql::parser::types::{
    EnumType, EnumValueDefinition, TypeDefinition, TypeKind, TypeSystemDefinition,
};
use async_graphql::{Name, Positioned};
use schemars::schema::Schema;

#[derive(Debug)]
pub struct EnumVariant {
    pub value: String,
    pub description: Option<Positioned<String>>,
}

impl EnumVariant {
    pub fn new(value: String) -> Self {
        Self { value, description: None }
    }
}

#[derive(Debug)]
pub struct EnumValue {
    pub variants: Vec<EnumVariant>,
    pub description: Option<Positioned<String>>,
}

use crate::common::{get_description, pos};

pub fn into_enum_definition(enum_value: EnumValue, name: &str) -> TypeSystemDefinition {
    let mut enum_value_definition = vec![];
    for enum_variant in enum_value.variants {
        let formatted_value: String = enum_variant
            .value
            .to_string()
            .chars()
            .filter(|ch| ch != &'"')
            .collect();
        enum_value_definition.push(pos(EnumValueDefinition {
            value: pos(Name::new(formatted_value)),
            description: enum_variant.description,
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
            let description =
                get_description(schema_object).map(|description| pos(description.to_owned()));

            // if it has enum_values then it's raw enum
            if let Some(enum_values) = &schema_object.enum_values {
                return Some(EnumValue {
                    variants: enum_values
                        .iter()
                        .map(|val| EnumVariant::new(val.to_string()))
                        .collect::<Vec<_>>(),
                    description,
                });
            }

            // in case enum has description docs for the variants they will be generated
            // as schema with `one_of` entry, where every enum variant is separate enum
            // entry
            if let Some(subschema) = &schema_object.subschemas {
                if let Some(one_ofs) = &subschema.one_of {
                    let variants = one_ofs
                        .iter()
                        .filter_map(|one_of| {
                            // try to parse one_of value as enum
                            into_enum_value(one_of).and_then(|mut en| {
                                // if it has only single variant it's our high-level enum
                                if en.variants.len() == 1 {
                                    Some(EnumVariant {
                                        value: en.variants.pop().unwrap().value,
                                        description: en.description,
                                    })
                                } else {
                                    None
                                }
                            })
                        })
                        .collect::<Vec<_>>();

                    if !variants.is_empty() {
                        return Some(EnumValue { variants, description });
                    }
                }
            }

            None
        }
        _ => None,
    }
}
