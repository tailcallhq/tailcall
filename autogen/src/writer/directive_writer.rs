use crate::types::*;
use anyhow::Result;
use schemars::schema::{Schema, SchemaObject};
use std::{
    collections::{BTreeMap, HashSet},
    io::Write,
};
use tailcall::config;

use crate::{Entity, ExtraTypes, IndentedWriter, DIRECTIVE_ALLOW_LIST};

fn directive_allow_list_lookup(name: &str) -> Option<(&'static str, &'static Vec<Entity>, bool)> {
    for (nm, entity, is_repeatable) in DIRECTIVE_ALLOW_LIST.iter() {
        if name.to_lowercase() == nm.to_lowercase() {
            return Some((nm, entity, *is_repeatable));
        }
    }
    None
}

pub struct Directives {
    written_directives: HashSet<String>,
}

impl Directives {
    pub fn new(written_directives: HashSet<String>) -> Self {
        Directives { written_directives }
    }

    // Write all directives: parser from RootSchema
    pub fn write(
        &mut self,
        writer: &mut IndentedWriter<impl Write>,
        extra_it: &mut BTreeMap<String, ExtraTypes>,
    ) -> Result<()> {
        let schema = schemars::schema_for!(config::Config);
        let defs: BTreeMap<String, Schema> = schema.definitions;

        for (name, schema) in defs.iter() {
            let schema = schema.clone().into_object();
            self.write_directive(name.clone(), schema, &defs, writer, extra_it)?;
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    // Write directive
    fn write_directive(
        &mut self,
        name: String,
        schema: SchemaObject,
        defs: &BTreeMap<String, Schema>,
        writer: &mut IndentedWriter<impl Write>,
        extra_it: &mut BTreeMap<String, ExtraTypes>,
    ) -> std::io::Result<()> {
        let (name, entities, is_repeatable) = match directive_allow_list_lookup(&name) {
            Some(entity) => entity,
            None => return Ok(()),
        };

        if self.written_directives.contains(name) {
            return Ok(());
        }

        let description = schema
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.description.as_ref());
        // description
        crate::writer::common::write_description(writer, description)?;

        // start write body
        write!(writer, "directive @{}", name)?;
        if let Some(properties) = schema.object.map(|object| object.properties) {
            let mut properties_iter = properties.into_iter();

            let mut close_param = false;
            if let Some((name, property)) = properties_iter.next() {
                writeln!(writer, "(")?;
                writer.indent();
                crate::writer::common::write_property(
                    writer,
                    name,
                    property,
                    defs,
                    extra_it,
                )?;
                close_param = true;
            }
            for (name, property) in properties_iter {
                crate::writer::common::write_property(
                    writer,
                    name,
                    property,
                    defs,
                    extra_it,
                )?;
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
        self.written_directives.insert(name.to_string());

        Ok(())
    }
}
