use crate::types::*;

use crate::writer::IndentedWriter;
use anyhow::Result;
use schemars::schema::{
    ArrayValidation, InstanceType, ObjectValidation, Schema, SchemaObject, SingleOrVec,
};
use schemars::Map;
use std::collections::{BTreeMap, HashSet};
use std::io::Write;
use tailcall::config;
use tailcall::scalar::CUSTOM_SCALARS;

fn write_description(
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

fn write_instance_type(
    writer: &mut IndentedWriter<impl Write>,
    typ: &InstanceType,
) -> std::io::Result<()> {
    match typ {
        &InstanceType::Integer => write!(writer, "Int"),
        x => write!(writer, "{x:?}"),
    }
}

fn write_reference(
    writer: &mut IndentedWriter<impl Write>,
    reference: &str,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> std::io::Result<()> {
    let mut nm = reference.split('/').last().unwrap().to_string();
    first_char_to_upper(&mut nm);
    extra_it.insert(nm.clone(), ExtraTypes::Schema);
    write!(writer, "{nm}")
}

fn first_char_to_upper(name: &mut String) {
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
fn write_type(
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
#[allow(clippy::too_many_arguments)]
fn write_field(
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
fn write_input_type(
    writer: &mut IndentedWriter<impl Write>,
    name: String,
    typ: SchemaObject,
    defs: &BTreeMap<String, Schema>,
    scalar: &mut HashSet<String>,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
    types_added: &mut HashSet<String>,
) -> std::io::Result<()> {
    let name = match input_allow_list_lookup(&name, extra_it) {
        Some(name) => name,
        None => return Ok(()),
    };

    if types_added.contains(name) {
        return Ok(());
    } else {
        types_added.insert(name.to_string());
    }

    let description = typ
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.description.as_ref());
    write_description(writer, description)?;
    if let Some(obj) = typ.object {
        if obj.properties.is_empty() {
            scalar.insert(name.to_string());
            return Ok(());
        }
        writeln!(writer, "input {name} {{")?;
        writer.indent();
        for (name, property) in obj.properties.into_iter() {
            let property = property.into_object();
            let description = property
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.description.as_ref());
            write_description(writer, description)?;
            write_field(writer, name, property, defs, extra_it)?;
        }
        writer.unindent();
        writeln!(writer, "}}")?;
    } else if let Some(enm) = typ.enum_values {
        writeln!(writer, "enum {name} {{")?;
        writer.indent();
        for val in enm {
            let val: String = format!("{val}").chars().filter(|ch| ch != &'"').collect();
            writeln!(writer, "{val}")?;
        }
        writer.unindent();
        writeln!(writer, "}}")?;
    } else if let Some(list) = typ.subschemas.as_ref().and_then(|ss| ss.any_of.as_ref()) {
        if list.is_empty() {
            scalar.insert(name.to_string());
            return Ok(());
        }
        writeln!(writer, "input {name} {{")?;
        writer.indent();
        for property in list {
            let property = property.clone().into_object();
            let description = property
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.description.as_ref());
            write_description(writer, description)?;
            if let Some(obj) = property.object {
                for (name, schema) in obj.properties {
                    write_field(writer, name, schema.into_object(), defs, extra_it)?;
                }
            }
        }
        writer.unindent();
        writeln!(writer, "}}")?;
    } else if let Some(list) = typ.subschemas.as_ref().and_then(|ss| ss.one_of.as_ref()) {
        if list.is_empty() {
            scalar.insert(name.to_string());
            return Ok(());
        }
        writeln!(writer, "input {name} {{")?;
        writer.indent();
        for property in list {
            if let Some(obj) = property.clone().into_object().object {
                for (name, schema) in obj.properties {
                    write_field(writer, name, schema.into_object(), defs, extra_it)?;
                }
            }
        }
        writer.unindent();
        writeln!(writer, "}}")?;
    } else if let Some(SingleOrVec::Single(item)) = typ.array.and_then(|arr| arr.items) {
        if let Some(name) = item.into_object().reference {
            writeln!(writer, "{name}")?;
        } else {
            scalar.insert(name.to_string());
        }
    }

    Ok(())
}
#[allow(clippy::too_many_arguments)]
fn write_property(
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

fn directive_allow_list_lookup(name: &str) -> Option<(&'static str, &'static Vec<Entity>, bool)> {
    for (nm, entity, is_repeatable) in DIRECTIVE_ALLOW_LIST.iter() {
        if name.to_lowercase() == nm.to_lowercase() {
            return Some((nm, entity, *is_repeatable));
        }
    }
    None
}

fn input_allow_list_lookup<'a>(
    name: &'a str,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> Option<&'a str> {
    for nm in OBJECT_WHITELIST.iter() {
        if name.to_lowercase() == nm.to_lowercase() {
            return Some(*nm);
        }
    }

    if extra_it.contains_key(name) {
        return Some(name);
    }

    None
}
#[allow(clippy::too_many_arguments)]
fn write_directive(
    writer: &mut IndentedWriter<impl Write>,
    name: String,
    schema: SchemaObject,
    defs: &BTreeMap<String, Schema>,
    written_directives: &mut HashSet<String>,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> std::io::Result<()> {
    let (name, entities, is_repeatable) = match directive_allow_list_lookup(&name) {
        Some(entity) => entity,
        None => return Ok(()),
    };

    if written_directives.contains(name) {
        return Ok(());
    }

    let description = schema
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.description.as_ref());
    write_description(writer, description)?;

    write!(writer, "directive @{}", name)?;
    if let Some(properties) = schema.object.map(|object| object.properties) {
        let mut properties_iter = properties.into_iter();

        let mut close_param = false;
        if let Some((name, property)) = properties_iter.next() {
            writeln!(writer, "(")?;
            writer.indent();
            write_property(writer, name, property, defs, extra_it)?;
            close_param = true;
        }
        for (name, property) in properties_iter {
            write_property(writer, name, property, defs, extra_it)?;
        }
        if close_param {
            writer.unindent();
            write!(writer, ")")?;
        }
    }

    if is_repeatable {
        write!(writer, " repeatable ")?;
    }

    entities.to_graphql(writer)?;
    written_directives.insert(name.to_string());

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_array_validation(
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
fn write_object_validation(
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

pub struct Document<W: Write> {
    writer: IndentedWriter<W>,
}
impl<W: Write> Document<W> {
    pub fn new(dest: W) -> Self {
        let writer = IndentedWriter::new(dest);
        Document { writer }
    }

    pub fn print(&mut self) -> anyhow::Result<()> {
        self.update_gql()?;
        Ok(())
    }

    pub fn update_gql(&mut self) -> Result<()> {
        self.generate_rc_file()?;
        Ok(())
    }

    fn generate_rc_file(&mut self) -> Result<()> {
        let mut written_directives = HashSet::new();

        let mut extra_it = BTreeMap::new();

        self.write_all_directives(&mut written_directives, &mut extra_it)?;
        self.write_all_input_types(extra_it)?;

        Ok(())
    }

    fn write_all_directives(
        &mut self,
        written_directives: &mut HashSet<String>,
        extra_it: &mut BTreeMap<String, ExtraTypes>,
    ) -> Result<()> {
        let schema = schemars::schema_for!(config::Config);

        let defs: BTreeMap<String, Schema> = schema.definitions;
        for (name, schema) in defs.iter() {
            let schema = schema.clone().into_object();
            write_directive(
                &mut self.writer,
                name.clone(),
                schema,
                &defs,
                written_directives,
                extra_it,
            )?;
        }

        Ok(())
    }

    fn write_all_input_types(
        &mut self,
        mut extra_it: BTreeMap<String, ExtraTypes>,
    ) -> std::io::Result<()> {
        let schema = schemars::schema_for!(config::Config);

        let scalar = CUSTOM_SCALARS
            .iter()
            .map(|(k, v)| (k.clone(), v.scalar()))
            .collect::<Map<String, Schema>>();

        let mut scalar_defs = BTreeMap::new();

        for (name, obj) in scalar.iter() {
            let scalar_definition = obj
                .clone()
                .into_object()
                .object
                .as_ref()
                .and_then(|a| a.properties.get(name))
                .and_then(|a| a.clone().into_object().metadata)
                .and_then(|a| a.description);

            if let Some(scalar_definition) = scalar_definition {
                scalar_defs.insert(name.clone(), scalar_definition);
            }
        }

        let defs = schema.definitions;

        let mut scalar = scalar
            .keys()
            .map(|v| v.to_string())
            .collect::<HashSet<String>>();

        let mut types_added = HashSet::new();
        for (name, input_type) in defs.iter() {
            let mut name = name.clone();
            first_char_to_upper(&mut name);
            write_input_type(
                &mut self.writer,
                name,
                input_type.clone().into_object(),
                &defs,
                &mut scalar,
                &mut extra_it,
                &mut types_added,
            )?;
        }

        let mut new_extra_it = BTreeMap::new();

        for (name, extra_type) in extra_it.into_iter() {
            match extra_type {
                ExtraTypes::Schema => {
                    if let Some(schema) = defs.get(&name).cloned() {
                        write_input_type(
                            &mut self.writer,
                            name,
                            schema.into_object(),
                            &defs,
                            &mut scalar,
                            &mut new_extra_it,
                            &mut types_added,
                        )?
                    }
                }
                ExtraTypes::ObjectValidation(obj_valid) => write_object_validation(
                    &mut self.writer,
                    name,
                    obj_valid,
                    &defs,
                    &mut new_extra_it,
                )?,
            }
        }

        let mut scalar_vector: Vec<String> = Vec::from_iter(scalar);
        scalar_vector.sort();

        for name in scalar_vector {
            if scalar_defs.contains_key(&name) {
                let def = scalar_defs.get(&name).unwrap();
                writeln!(&mut self.writer, "\"\"\"")?;
                writeln!(&mut self.writer, "{def}")?;
                writeln!(&mut self.writer, "\"\"\"")?;
                writeln!(&mut self.writer, "scalar {name}")?;
            } else {
                writeln!(&mut self.writer, "scalar {name}")?;
            }
        }

        Ok(())
    }
}
