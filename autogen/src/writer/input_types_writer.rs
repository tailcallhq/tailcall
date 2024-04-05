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
        extra_it: BTreeMap<String, ExtraTypes>,
    ) -> Result<()> {
        let input_types_str = self.write_all_input_types(extra_it);
        Ok(())
    }

    fn write_all_input_types(&mut self, mut extra_it: BTreeMap<String, ExtraTypes>) -> String {
        let mut list = vec![];
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
            name = uppercase_first(&name);
            list.push(self.write_input_type(
                name,
                input_type.clone().into_object(),
                &defs,
                &mut scalar,
                &mut extra_it,
                &mut types_added,
            ));
        }

        let mut new_extra_it = BTreeMap::new();

        for (name, extra_type) in extra_it.into_iter() {
            match extra_type {
                ExtraTypes::Schema => {
                    if let Some(schema) = defs.get(&name).cloned() {
                        list.push(self.write_input_type(
                            name,
                            schema.into_object(),
                            &defs,
                            &mut scalar,
                            &mut new_extra_it,
                            &mut types_added,
                        ))
                    }
                }
                ExtraTypes::ObjectValidation(obj_valid) => {
                    let object_str =
                        write_object_validation(name, obj_valid, &defs, &mut new_extra_it);
                    list.push(object_str);
                }
            }
        }

        let mut scalar_vector: Vec<String> = Vec::from_iter(scalar);
        scalar_vector.sort();

        for name in scalar_vector {
            if scalar_defs.contains_key(&name) {
                let def = scalar_defs.get(&name).unwrap();
                list.push(format!("\"\"\"\n"));
                list.push(format!("{def}\n"));
                list.push(format!("\"\"\"\n"));
                list.push(format!("scalar {name}\n"));
            } else {
                list.push(format!("scalar {name}\n"));
            }
        }

        list.join("")
    }

    #[allow(clippy::too_many_arguments)]
    fn write_input_type(
        &mut self,
        name: String,
        typ: SchemaObject,
        defs: &BTreeMap<String, Schema>,
        scalar: &mut HashSet<String>,
        extra_it: &mut BTreeMap<String, ExtraTypes>,
        types_added: &mut HashSet<String>,
    ) -> String {
        let name = match input_allow_list_lookup(&name, extra_it) {
            Some(name) => name,
            None => return "".to_string(),
        };

        if types_added.contains(name) {
            return "".to_string();
        } else {
            types_added.insert(name.to_string());
        }

        if let Some(description) = typ
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.description.as_ref())
        {
            description_str(description.clone());
        }

        let mut list: Vec<String> = vec![];

        if let Some(obj) = typ.object {
            if obj.properties.is_empty() {
                scalar.insert(name.to_string());
                return "".to_string();
            }

            list.push(format!("input {name} {{\n"));

            for (name, property) in obj.properties.into_iter() {
                let property = property.into_object();
                if let Some(description) = property
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.description.as_ref())
                {
                    list.push(format!("\t{}", description_str(description.clone())));
                }
                list.push(write_field(name, property, defs, extra_it));
            }
            list.push(format!("}}\n"));
        } else if let Some(enm) = typ.enum_values {
            list.push(format!("enum {name} {{\n"));
            for val in enm {
                let val: String = format!("{val}").chars().filter(|ch| ch != &'"').collect();
                list.push(format!("\t{val}"));
            }
            list.push(format!("}}\n"));
        } else if let Some(list_schema) = typ.subschemas.as_ref().and_then(|ss| ss.any_of.as_ref())
        {
            if list_schema.is_empty() {
                scalar.insert(name.to_string());
                return "".to_string();
            }

            list.push(format!("input {name} {{\n"));

            for p in list_schema {
                let property = p.clone().into_object();
                if let Some(description) = property
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.description.as_ref())
                {
                    list.push(format!("\t{}", description_str(description.clone())));
                }

                if let Some(obj) = property.object {
                    for (name, _schema) in obj.properties {
                        list.push(format!(
                            "\t{}",
                            write_field(name, p.clone().into_object(), defs, extra_it)
                        ));
                    }
                }
            }
            list.push(format!("}}\n"));
        } else if let Some(list_schema) = typ.subschemas.as_ref().and_then(|ss| ss.one_of.as_ref())
        {
            if list_schema.is_empty() {
                scalar.insert(name.to_string());
                return "".to_string();
            }

            list.push(format!("input {name} {{\n"));

            for property in list_schema {
                if let Some(obj) = property.clone().into_object().object {
                    for (name, schema) in obj.properties {
                        list.push(format!(
                            "\t{}",
                            write_field(name, schema.into_object(), defs, extra_it)
                        ));
                    }
                }
            }
            list.push(format!("}}\n"));
        } else if let Some(SingleOrVec::Single(item)) = typ.array.and_then(|arr| arr.items) {
            if let Some(name) = item.into_object().reference {
                list.push(format!("{name}"));
            } else {
                scalar.insert(name.to_string());
            }
        }

        list.join("")
    }
}
