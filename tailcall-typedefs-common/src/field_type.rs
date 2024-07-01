use async_graphql_parser::types::{BaseType, Type};
use async_graphql_value::Name;
use schemars::schema::{
    ArrayValidation, InstanceType, ObjectValidation, Schema, SchemaObject, SingleOrVec,
};

pub fn get_field_type(name: String, schema: SchemaObject) -> Type {
    if let Some(instance_type) = &schema.instance_type {
        match instance_type {
            SingleOrVec::Single(typ) => match **typ {
                InstanceType::Null
                | InstanceType::Boolean
                | InstanceType::Number
                | InstanceType::String
                | InstanceType::Integer => Type {
                    nullable: false,
                    base: BaseType::Named(Name::new(get_instace_type_name(&*typ))),
                },
                _ => determine_type_from_schema(name, &schema),
            },
            SingleOrVec::Vec(typ) => match typ.first().unwrap() {
                InstanceType::Null
                | InstanceType::Boolean
                | InstanceType::Number
                | InstanceType::String
                | InstanceType::Integer => Type {
                    nullable: false,
                    base: BaseType::Named(Name::new(get_instace_type_name(typ.first().unwrap()))),
                },
                _ => determine_type_from_schema(name, &schema),
            },
        }
    } else {
        return determine_type_from_schema(name, &schema);
    }
}

fn determine_type_from_schema(name: String, schema: &SchemaObject) -> Type {
    if let Some(arr_valid) = &schema.array {
        return get_type_from_arr_valid(name, &arr_valid);
    }

    if let Some(typ) = &schema.object {
        return get_type_from_object_valid(name, &typ);
    }

    if let Some(subschema) = schema.subschemas.clone().into_iter().next() {
        let list = subschema.any_of.or(subschema.all_of).or(subschema.one_of);

        if let Some(list) = list {
            if let Some(Schema::Object(obj)) = list.first() {
                if let Some(reference) = &obj.reference {
                    return get_type_from_reference(reference);
                }
            }
        }
    }

    if let Some(reference) = &schema.reference {
        return get_type_from_reference(reference);
    }

    Type { nullable: true, base: BaseType::Named(Name::new("JSON")) }
}

fn get_type_from_reference(reference: &str) -> Type {
    let name = reference.split('/').last().unwrap().to_string();
    Type { nullable: true, base: BaseType::Named(Name::new(name)) }
}

fn get_type_from_arr_valid(name: String, array_valid: &ArrayValidation) -> Type {
    if let Some(items) = &array_valid.items {
        match items {
            SingleOrVec::Single(schema) => Type {
                nullable: true,
                base: BaseType::List(Box::new(get_field_type(name, schema.clone().into_object()))),
            },
            SingleOrVec::Vec(schemas) => Type {
                nullable: true,
                base: BaseType::List(Box::new(get_field_type(
                    name,
                    schemas[0].clone().into_object(),
                ))),
            },
        }
    } else {
        return Type { nullable: true, base: BaseType::Named(Name::new("JSON")) };
    }
}

fn get_type_from_object_valid(name: String, typ: &ObjectValidation) -> Type {
    if !typ.properties.is_empty() {
        Type { nullable: true, base: BaseType::Named(Name::new(name)) }
    } else {
        Type { nullable: true, base: BaseType::Named(Name::new("JSON")) }
    }
}

fn get_instace_type_name(typ: &InstanceType) -> String {
    match typ {
        &InstanceType::Integer => "Int".to_string(),
        _ => format!("{:?}", typ),
    }
}
