use async_graphql::parser::types::{
    ConstDirective, EnumType, EnumValueDefinition, FieldDefinition, InputObjectType,
    InputValueDefinition, InterfaceType, ObjectType, SchemaDefinition, ServiceDocument,
    TypeDefinition, TypeKind, TypeSystemDefinition, UnionType,
};
use async_graphql::{Name, Positioned};
use async_graphql_value::ConstValue;
use tailcall_valid::Validator;

use super::blueprint;
use super::directive::{to_const_directive, Directive};
use crate::core::blueprint::{Blueprint, Definition};
use crate::core::pos;

fn to_directives(directives: &[Directive]) -> Vec<Positioned<ConstDirective>> {
    directives
        .iter()
        // TODO: conversion if fallible but the overall conversion from blueprint should infallible
        .filter_map(|directive| to_const_directive(directive).to_result().ok())
        .map(pos)
        .collect()
}

fn to_args(args: &[blueprint::InputFieldDefinition]) -> Vec<Positioned<InputValueDefinition>> {
    args.iter()
        .map(|input| {
            let of_type = &input.of_type;

            pos(InputValueDefinition {
                description: None,
                name: pos(Name::new(&input.name)),
                ty: pos(of_type.into()),
                default_value: input
                    .default_value
                    .clone()
                    .and_then(|value| ConstValue::from_json(value).ok())
                    .map(pos),
                directives: Vec::new(),
            })
        })
        .collect()
}

fn to_fields(fields: &[blueprint::FieldDefinition]) -> Vec<Positioned<FieldDefinition>> {
    fields
        .iter()
        .map(|field| {
            let of_type = &field.of_type;
            let arguments = to_args(&field.args);

            pos(FieldDefinition {
                description: None,
                name: pos(Name::new(&field.name)),
                arguments,
                ty: pos(of_type.into()),
                directives: to_directives(&field.directives),
            })
        })
        .collect()
}

fn to_definition(def: &Definition) -> TypeSystemDefinition {
    let kind = match def {
        Definition::Object(def) => TypeKind::Object(ObjectType {
            implements: def
                .implements
                .iter()
                .map(|name| pos(Name::new(name)))
                .collect(),
            fields: to_fields(&def.fields),
        }),
        Definition::Interface(def) => TypeKind::Interface(InterfaceType {
            implements: def
                .implements
                .iter()
                .map(|name| pos(Name::new(name)))
                .collect(),
            fields: to_fields(&def.fields),
        }),
        Definition::InputObject(def) => TypeKind::InputObject(InputObjectType {
            fields: def
                .fields
                .iter()
                .map(|input| {
                    let of_type = &input.of_type;

                    pos(InputValueDefinition {
                        description: None,
                        name: pos(Name::new(&input.name)),
                        ty: pos(of_type.into()),
                        default_value: input
                            .default_value
                            .clone()
                            .and_then(|value| ConstValue::from_json(value).ok())
                            .map(pos),
                        directives: Vec::new(),
                    })
                })
                .collect(),
        }),
        Definition::Scalar(_) => TypeKind::Scalar,
        Definition::Enum(def) => TypeKind::Enum(EnumType {
            values: def
                .enum_values
                .iter()
                .map(|variant| {
                    pos(EnumValueDefinition {
                        description: None,
                        value: pos(Name::new(&variant.name)),
                        directives: Vec::new(),
                    })
                })
                .collect(),
        }),
        Definition::Union(def) => TypeKind::Union(UnionType {
            members: def.types.iter().map(|name| pos(Name::new(name))).collect(),
        }),
    };

    TypeSystemDefinition::Type(pos(TypeDefinition {
        extend: false,
        description: None,
        name: pos(Name::new(def.name())),
        directives: to_directives(def.directives()),
        kind,
    }))
}

impl From<&Blueprint> for ServiceDocument {
    fn from(blueprint: &Blueprint) -> Self {
        let mut definitions = Vec::new();

        definitions.push(TypeSystemDefinition::Schema(pos(SchemaDefinition {
            extend: false,
            directives: Vec::new(),
            query: Some(pos(Name::new(&blueprint.schema.query))),
            mutation: blueprint
                .schema
                .mutation
                .as_ref()
                .map(|mutation| pos(Name::new(mutation))),
            subscription: None,
        })));

        for def in &blueprint.definitions {
            definitions.push(to_definition(def))
        }

        Self { definitions }
    }
}
