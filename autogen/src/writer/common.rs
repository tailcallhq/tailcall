use crate::types::*;
use crate::writer::IndentedWriter;
use schemars::schema::{
    ArrayValidation, InstanceType, ObjectValidation, Schema, SchemaObject, SingleOrVec,
};
use std::collections::BTreeMap;
use std::io::Write;

pub fn description_str(description: String) -> String {
    let description: String = description.chars().filter(|ch| ch != &'\n').collect();
    let line_breaker = LineBreaker::new(&description, 80);

    let mut list: Vec<String> = vec![];
    list.push("\"\"\"".to_string());
    for line in line_breaker {
        let l = format!("{line}");
        list.push(l);
    }
    list.push("\"\"\"".to_string());
    list.join("\n")
}

#[allow(clippy::too_many_arguments)]
pub fn property_str(
    name: String,
    property: Schema,
    defs: &BTreeMap<String, Schema>,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> String {
    let property = property.into_object();
    let description = property
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.description.as_ref());
    let mut list = vec![];
    if let Some(d) = description {
        list.push(description_str(d.clone()));
    }
    description_str(write_field(name, property, defs, extra_it));
    list.join("\n")
}

#[allow(clippy::too_many_arguments)]
pub fn write_field(
    field_name: String,
    schema: SchemaObject,
    defs: &BTreeMap<String, Schema>,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> String {
    format!(
        "{name}: {t}",
        name = field_name,
        t = write_type(field_name.clone(), schema, defs, extra_it)
    )
}

pub fn uppercase_first(name: &str) -> String {
    if let Some(first_char) = name.chars().next() {
        // Remove the first character and make it uppercase
        let first_char_upper = first_char.to_uppercase().to_string();

        // Remove the first character from the original string
        let mut chars = name.chars();
        chars.next();

        // Replace the original string with the new one
        return first_char_upper + chars.as_str();
    }
    return "".to_string();
}

#[allow(clippy::too_many_arguments)]
pub fn write_type(
    name: String,
    schema: SchemaObject,
    defs: &BTreeMap<String, Schema>,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> String {
    match schema.instance_type {
        Some(SingleOrVec::Single(typ))
            if matches!(
                *typ,
                InstanceType::Null
                    | InstanceType::Boolean
                    | InstanceType::Number
                    | InstanceType::String
                    | InstanceType::Integer
            ) =>
        {
            format!("{}!", instance_type(&typ))
        }
        Some(SingleOrVec::Vec(typ))
            if matches!(
                typ.first().unwrap(),
                InstanceType::Null
                    | InstanceType::Boolean
                    | InstanceType::Number
                    | InstanceType::String
                    | InstanceType::Integer
            ) =>
        {
            format!("{}!", instance_type(typ.first().unwrap()))
        }
        _ => {
            if let Some(arr_valid) = schema.array.clone() {
                write_array_validation(name, *arr_valid, defs, extra_it)
            } else if let Some(typ) = schema.object.clone() {
                if !typ.properties.is_empty() {
                    let n = uppercase_first(&name);
                    extra_it.insert(name, ExtraTypes::ObjectValidation(*typ));
                    format!("{}", n.clone())
                } else {
                    format!("JSON")
                }
            } else if let Some(sub_schema) = schema.subschemas.clone().into_iter().next() {
                let list = if let Some(list) = sub_schema.any_of {
                    list
                } else if let Some(list) = sub_schema.all_of {
                    list
                } else if let Some(list) = sub_schema.one_of {
                    list
                } else {
                    return format!("JSON");
                };
                let first = list.first().unwrap();
                match first {
                    Schema::Object(obj) => {
                        let nm = reference_str(&obj.reference.clone().unwrap());
                        extra_it.insert(nm.clone(), ExtraTypes::Schema);
                        return format!("{}", nm);
                    }
                    _ => panic!(),
                }
            } else if let Some(name) = schema.reference {
                let nm = reference_str(&name);
                extra_it.insert(nm.clone(), ExtraTypes::Schema);
                return format!("{}", nm);
            } else {
                return format!("JSON");
            }
        }
    }
}

pub fn instance_type(typ: &InstanceType) -> String {
    match typ {
        &InstanceType::Integer => "Int".to_string(),
        _x => "{x:?}".to_string(),
    }
}

// refernce
pub fn reference_str(reference: &str) -> String {
    let nm = reference.split('/').last().unwrap().to_string();
    format!("{nm}", nm = uppercase_first(&nm))
}

#[allow(clippy::too_many_arguments)]
pub fn write_array_validation(
    name: String,
    arr_valid: ArrayValidation,
    defs: &BTreeMap<String, Schema>,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> String {
    let mut list: Vec<String> = vec![];
    list.push("[".to_string());
    if let Some(SingleOrVec::Single(schema)) = arr_valid.items {
        list.push(write_type(name, schema.into_object(), defs, extra_it));
    } else if let Some(SingleOrVec::Vec(schemas)) = arr_valid.items {
        list.push(write_type(
            name,
            schemas[0].clone().into_object(),
            defs,
            extra_it,
        ));
    } else {
        list.push("JSON".to_string());
    }
    list.push("]".to_string());
    list.join("")
}

#[allow(clippy::too_many_arguments)]
pub fn write_object_validation(
    name: String,
    obj_valid: ObjectValidation,
    defs: &BTreeMap<String, Schema>,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> String {
    if !obj_valid.properties.is_empty() {
        let mut list = vec![];
        list.push("input {name} {{");
        for (name, property) in obj_valid.properties {
            let t = property_str(name, property, defs, extra_it);
            list.push("\t{t}");
        }
        list.push("}}");
        list.join("")
    } else {
        "".to_string()
    }
}
