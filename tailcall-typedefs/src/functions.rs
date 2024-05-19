// Functions for generating and manipulating GraphQL schema components.

use std::collections::{BTreeMap, HashSet};
use anyhow::Result;
use async_graphql::ServiceDocument;
use schemars::schema::Schema;
use tailcall::core::config::Config;
use tailcall::core::scalar::CUSTOM_SCALARS;
use crate::entity::Entity;
use crate::to_graphql::ToGraphql;

// Generates the complete GraphQL schema and writes to a file.
pub fn generate_rc_file(doc: &mut ServiceDocument) -> Result<()> {
    let mut written_directives = HashSet::new();
    let mut extra_it = BTreeMap::new();

    write_all_directives(doc, &mut written_directives, &mut extra_it)?;
    write_all_input_types(doc, extra_it)?;

    Ok(())
}

fn write_all_directives(
    doc: &mut ServiceDocument,
    written_directives: &mut HashSet<String>,
    extra_it: &mut BTreeMap<String, ExtraTypes>,
) -> Result<()> {
    let schema = schemars::schema_for!(Config);

    let defs: BTreeMap<String, Schema> = schema.definitions;
    for (name, schema) in defs.iter() {
        let schema = schema.clone().into_object();
        write_directive(
            doc,
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
    doc: &mut ServiceDocument,
    mut extra_it: BTreeMap<String, ExtraTypes>,
) -> Result<()> {
    let schema = schemars::schema_for!(Config);

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
            doc,
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
                        doc,
                        name,
                        schema.into_object(),
                        &defs,
                        &mut scalar,
                        &mut new_extra_it,
                        &mut types_added,
                    )?
                }
            }
            ExtraTypes::ObjectValidation(obj_valid) => {
                write_object_validation(doc, name, obj_valid, &defs, &mut new_extra_it)?
            }
        }
    }

    let mut scalar_vector: Vec<String> = Vec::from_iter(scalar);
    scalar_vector.sort();

    for name in scalar_vector {
        if scalar_defs.contains_key(&name) {
            let def = scalar_defs.get(&name).unwrap();
            doc.add_scalar(&name, Some(def));
        } else {
            doc.add_scalar(&name, None);
        }
    }

    Ok(())
}