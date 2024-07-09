use async_graphql::parser::types::{
    BaseType, InputObjectType, InputValueDefinition, Type, TypeDefinition, TypeKind,
    TypeSystemDefinition,
};
use async_graphql::{Name, Positioned};
use schemars::schema::{
    ArrayValidation, InstanceType, ObjectValidation, Schema, SchemaObject, SingleOrVec,
};

use crate::common::{first_char_to_upper, get_description, pos};

pub trait InputDefinition {
    fn input_definition() -> TypeSystemDefinition;
}

pub fn into_input_definition(schema: SchemaObject, name: &str) -> TypeSystemDefinition {
    let description = get_description(&schema);

    TypeSystemDefinition::Type(pos(TypeDefinition {
        name: pos(Name::new(name)),
        kind: TypeKind::InputObject(InputObjectType {
            fields: into_input_value_definition(&schema),
        }),
        description: description.map(|inner| pos(inner.clone())),
        directives: vec![],
        extend: false,
    }))
}

pub fn into_input_value_definition(schema: &SchemaObject) -> Vec<Positioned<InputValueDefinition>> {
    let mut arguments_type = vec![];
    if let Some(subschema) = schema.subschemas.clone() {
        let list = subschema.any_of.or(subschema.all_of).or(subschema.one_of);
        if let Some(list) = list {
            for schema in list {
                let schema_object = schema.into_object();
                arguments_type.extend(build_arguments_type(&schema_object));
            }

            return arguments_type;
        }
    }

    build_arguments_type(schema)
}

fn build_arguments_type(schema: &SchemaObject) -> Vec<Positioned<InputValueDefinition>> {
    let mut arguments = vec![];
    if let Some(properties) = schema
        .object
        .as_ref()
        .map(|object| object.properties.clone())
    {
        for (name, property) in properties.into_iter() {
            let property = property.into_object();
            let description = get_description(&property);
            let definition = pos(InputValueDefinition {
                description: description.map(|inner| pos(inner.to_owned())),
                name: pos(Name::new(&name)),
                ty: pos(determine_input_value_type_from_schema(
                    name,
                    property.clone(),
                )),
                default_value: None,
                directives: Vec::new(),
            });

            arguments.push(definition);
        }
    }

    arguments
}

fn determine_input_value_type_from_schema(mut name: String, schema: SchemaObject) -> Type {
    first_char_to_upper(&mut name);
    if let Some(instance_type) = &schema.instance_type {
        match instance_type {
            SingleOrVec::Single(typ) => match **typ {
                InstanceType::Null
                | InstanceType::Boolean
                | InstanceType::Number
                | InstanceType::String
                | InstanceType::Integer => Type {
                    nullable: false,
                    base: BaseType::Named(Name::new(get_instance_type_name(typ))),
                },
                _ => determine_type_from_schema(name, &schema),
            },
            SingleOrVec::Vec(typ) => match typ.first().unwrap() {
                InstanceType::Null
                | InstanceType::Boolean
                | InstanceType::Number
                | InstanceType::String
                | InstanceType::Integer => Type {
                    nullable: true,
                    base: BaseType::Named(Name::new(get_instance_type_name(typ.first().unwrap()))),
                },
                _ => determine_type_from_schema(name, &schema),
            },
        }
    } else {
        determine_type_from_schema(name, &schema)
    }
}

fn determine_type_from_schema(name: String, schema: &SchemaObject) -> Type {
    if let Some(arr_valid) = &schema.array {
        return determine_type_from_arr_valid(name, arr_valid);
    }

    if let Some(typ) = &schema.object {
        return determine_type_from_object_valid(name, typ);
    }

    if let Some(subschema) = schema.subschemas.clone().into_iter().next() {
        let list = subschema.any_of.or(subschema.all_of).or(subschema.one_of);

        if let Some(list) = list {
            if let Some(Schema::Object(obj)) = list.first() {
                if let Some(reference) = &obj.reference {
                    return determine_type_from_reference(reference);
                }
            }
        }
    }

    if let Some(reference) = &schema.reference {
        return determine_type_from_reference(reference);
    }

    Type { nullable: true, base: BaseType::Named(Name::new("JSON")) }
}

fn determine_type_from_reference(reference: &str) -> Type {
    let mut name = reference.split('/').last().unwrap().to_string();
    first_char_to_upper(&mut name);
    Type { nullable: true, base: BaseType::Named(Name::new(name)) }
}

fn determine_type_from_arr_valid(name: String, array_valid: &ArrayValidation) -> Type {
    if let Some(items) = &array_valid.items {
        match items {
            SingleOrVec::Single(schema) => Type {
                nullable: true,
                base: BaseType::List(Box::new(determine_input_value_type_from_schema(
                    name,
                    schema.clone().into_object(),
                ))),
            },
            SingleOrVec::Vec(schemas) => Type {
                nullable: true,
                base: BaseType::List(Box::new(determine_input_value_type_from_schema(
                    name,
                    schemas[0].clone().into_object(),
                ))),
            },
        }
    } else {
        Type { nullable: true, base: BaseType::Named(Name::new("JSON")) }
    }
}

fn determine_type_from_object_valid(name: String, typ: &ObjectValidation) -> Type {
    if !typ.properties.is_empty() {
        Type { nullable: true, base: BaseType::Named(Name::new(name)) }
    } else {
        Type { nullable: true, base: BaseType::Named(Name::new("JSON")) }
    }
}

fn get_instance_type_name(typ: &InstanceType) -> String {
    match typ {
        &InstanceType::Integer => "Int".to_string(),
        _ => format!("{:?}", typ),
    }
}
