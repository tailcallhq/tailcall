use crate::types::*;
use crate::writer::IndentedWriter;
use schemars::schema::{
    ArrayValidation, InstanceType, ObjectValidation, Schema, SchemaObject, SingleOrVec,
};
use std::collections::BTreeMap;
use std::io::Write;

pub fn write_description(
    writer: &mut IndentedWriter<impl Write>,
    description: Option<&String>,
) -> std::io::Result<()> {
    if let Some(description) = description {
        let description: String = description.chars().filter(|ch| ch != &'\n').collect();
        let line_breaker = LineBreaker::new(&description, 80);
        writeln!(writer, "\"\"\"")?;
        for line in line_breaker {
            writeln!(writer, "{line}")?;
        }
        writeln!(writer, "\"\"\"")?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn write_property(
    writer: &mut IndentedWriter<impl Write>,
    name: String,
    property: Schema,
    defs: &BTreeMap<String, Schema>,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> std::io::Result<()> {
    let property = property.into_object();
    let description = property
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.description.as_ref());
    write_description(writer, description)?;
    write_field(writer, name, property, defs, extra_it)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn write_field(
    writer: &mut IndentedWriter<impl Write>,
    name: String,
    schema: SchemaObject,
    defs: &BTreeMap<String, Schema>,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> std::io::Result<()> {
    write!(writer, "{name}: ")?;
    write_type(writer, name, schema, defs, extra_it)?;
    writeln!(writer)
}

#[allow(clippy::too_many_arguments)]
pub fn first_char_to_upper(name: &mut String) {
    if let Some(first_char) = name.chars().next() {
        // Remove the first character and make it uppercase
        let first_char_upper = first_char.to_uppercase().to_string();

        // Remove the first character from the original string
        let mut chars = name.chars();
        chars.next();

        // Replace the original string with the new one
        *name = first_char_upper + chars.as_str();
    }
}

#[allow(clippy::too_many_arguments)]
pub fn write_type(
    writer: &mut IndentedWriter<impl Write>,
    name: String,
    schema: SchemaObject,
    defs: &BTreeMap<String, Schema>,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> std::io::Result<()> {
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
            write_instance_type(writer, &typ)?;
            write!(writer, "!")
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
            write_instance_type(writer, typ.first().unwrap())
        }
        _ => {
            if let Some(arr_valid) = schema.array.clone() {
                write_array_validation(writer, name, *arr_valid, defs, extra_it)
            } else if let Some(typ) = schema.object.clone() {
                if !typ.properties.is_empty() {
                    let mut name = name;
                    first_char_to_upper(&mut name);
                    write!(writer, "{name}")?;
                    extra_it.insert(name, ExtraTypes::ObjectValidation(*typ));
                    Ok(())
                } else {
                    write!(writer, "JSON")
                }
            } else if let Some(sub_schema) = schema.subschemas.clone().into_iter().next() {
                let list = if let Some(list) = sub_schema.any_of {
                    list
                } else if let Some(list) = sub_schema.all_of {
                    list
                } else if let Some(list) = sub_schema.one_of {
                    list
                } else {
                    write!(writer, "JSON")?;
                    return Ok(());
                };
                let first = list.first().unwrap();
                match first {
                    Schema::Object(obj) => {
                        write_reference(writer, &obj.reference.clone().unwrap(), extra_it)
                    }
                    _ => panic!(),
                }
            } else if let Some(name) = schema.reference {
                write_reference(writer, &name, extra_it)
            } else {
                write!(writer, "JSON")
            }
        }
    }
}

pub fn write_instance_type(
    writer: &mut IndentedWriter<impl Write>,
    typ: &InstanceType,
) -> std::io::Result<()> {
    match typ {
        &InstanceType::Integer => write!(writer, "Int"),
        x => write!(writer, "{x:?}"),
    }
}

// refernce
pub fn write_reference(
    writer: &mut IndentedWriter<impl Write>,
    reference: &str,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> std::io::Result<()> {
    let mut nm = reference.split('/').last().unwrap().to_string();
    first_char_to_upper(&mut nm);
    extra_it.insert(nm.clone(), ExtraTypes::Schema);
    write!(writer, "{nm}")
}

#[allow(clippy::too_many_arguments)]
pub fn write_array_validation(
    writer: &mut IndentedWriter<impl Write>,
    name: String,
    arr_valid: ArrayValidation,
    defs: &BTreeMap<String, Schema>,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> std::io::Result<()> {
    write!(writer, "[")?;
    if let Some(SingleOrVec::Single(schema)) = arr_valid.items {
        write_type(writer, name, schema.into_object(), defs, extra_it)?;
    } else if let Some(SingleOrVec::Vec(schemas)) = arr_valid.items {
        write_type(
            writer,
            name,
            schemas[0].clone().into_object(),
            defs,
            extra_it,
        )?;
    } else {
        write!(writer, "JSON")?;
    }
    write!(writer, "]")
}

#[allow(clippy::too_many_arguments)]
pub fn write_object_validation(
    writer: &mut IndentedWriter<impl Write>,
    name: String,
    obj_valid: ObjectValidation,
    defs: &BTreeMap<String, Schema>,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> std::io::Result<()> {
    if !obj_valid.properties.is_empty() {
        writeln!(writer, "input {name} {{")?;
        writer.indent();
        for (name, property) in obj_valid.properties {
            write_property(writer, name, property, defs, extra_it)?;
        }
        writer.unindent();
        writeln!(writer, "}}")
    } else {
        Ok(())
    }
}
