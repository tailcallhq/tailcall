use crate::types::*;
use anyhow::Result;
use schemars::schema::{Schema, SchemaObject};
use std::collections::{BTreeMap, HashSet};
use tailcall::config;

use crate::{Entity, ExtraTypes, DIRECTIVE_ALLOW_LIST};

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
    pub fn write(&mut self, extra_it: &mut BTreeMap<String, ExtraTypes>) -> Result<String> {
        let schema = schemars::schema_for!(config::Config);
        let defs: BTreeMap<String, Schema> = schema.definitions;
        let mut list: Vec<String> = vec![];
        for (name, schema) in defs.iter() {
            let schema = schema.clone().into_object();
            let directive = self.write_directive(name.clone(), schema, &defs, extra_it);
            list.push(directive);
        }

        Ok(list.join(""))
    }

    #[allow(clippy::too_many_arguments)]
    // Write directive
    fn write_directive(
        &mut self,
        name: String,
        schema: SchemaObject,
        defs: &BTreeMap<String, Schema>,
        extra_it: &mut BTreeMap<String, ExtraTypes>,
    ) -> String {
        let mut list = vec![];
        let (name, entities, is_repeatable) = match directive_allow_list_lookup(&name) {
            Some(entity) => entity,
            None => return "".to_string(),
        };

        if self.written_directives.contains(name) {
            return "".to_string();
        }

        if let Some(description) = schema
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.description.as_ref())
        {
            // description
            crate::writer::common::description_str(description.clone());
        }

        // start write body
        list.push(format!("directive @{}", name));
        if let Some(properties) = schema.object.map(|object| object.properties) {
            let mut properties_iter = properties.into_iter();

            let mut close_param = false;
            if let Some((name, property)) = properties_iter.next() {
                list.push(format!("(\n"));
                list.push(format!(
                    "\t{}",
                    crate::writer::common::property_str(name, property, defs, extra_it)
                ));
                close_param = true;
            }
            for (name, property) in properties_iter {
                list.push(format!(
                    "\t{}",
                    crate::writer::common::property_str(name, property, defs, extra_it)
                ));
            }
            if close_param {
                list.push(format!(")"));
            }
        }

        if is_repeatable {
            list.push(format!(" repeatable "));
        }

        list.push(entities.to_graphql().unwrap());
        self.written_directives.insert(name.to_string());

        list.join("")
    }
}
