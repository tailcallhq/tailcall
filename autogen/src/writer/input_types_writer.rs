use crate::types::*;
use crate::writer::common::*;
use crate::writer::IndentedWriter;
use anyhow::Result;
use schemars::schema::{Schema, SchemaObject, SingleOrVec};
use schemars::Map;
use std::collections::{BTreeMap, HashSet};
use std::io::Write;
use tailcall::config;
use tailcall::scalar::CUSTOM_SCALARS;

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

pub struct InputTypeWriter {}

impl InputTypeWriter {
    pub fn write(
        &mut self,
        writer: &mut IndentedWriter<impl Write>,
        mut extra_it: BTreeMap<String, ExtraTypes>,
    ) -> Result<()> {
        self.write_all_input_types(writer, extra_it);
        Ok(())
    }

    fn write_all_input_types(
        &mut self,
        writer: &mut IndentedWriter<impl Write>,
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
            self.write_input_type(
                name,
                input_type.clone().into_object(),
                &defs,
                &mut scalar,
                writer,
                &mut extra_it,
                &mut types_added,
            )?;
        }

        let mut new_extra_it = BTreeMap::new();

        for (name, extra_type) in extra_it.into_iter() {
            match extra_type {
                ExtraTypes::Schema => {
                    if let Some(schema) = defs.get(&name).cloned() {
                        self.write_input_type(
                            name,
                            schema.into_object(),
                            &defs,
                            &mut scalar,
                            writer,
                            &mut new_extra_it,
                            &mut types_added,
                        )?
                    }
                }
                ExtraTypes::ObjectValidation(obj_valid) => {
                    write_object_validation(writer, name, obj_valid, &defs, &mut new_extra_it)?
                }
            }
        }

        let mut scalar_vector: Vec<String> = Vec::from_iter(scalar);
        scalar_vector.sort();

        for name in scalar_vector {
            if scalar_defs.contains_key(&name) {
                let def = scalar_defs.get(&name).unwrap();
                writeln!(writer, "\"\"\"")?;
                writeln!(writer, "{def}")?;
                writeln!(writer, "\"\"\"")?;
                writeln!(writer, "scalar {name}")?;
            } else {
                writeln!(writer, "scalar {name}")?;
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn write_input_type(
        &mut self,
        name: String,
        typ: SchemaObject,
        defs: &BTreeMap<String, Schema>,
        scalar: &mut HashSet<String>,
        writer: &mut IndentedWriter<impl Write>,
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
}
