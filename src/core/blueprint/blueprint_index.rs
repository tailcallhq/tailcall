use std::collections::HashMap;

use super::{Blueprint, SchemaDefinition};
use crate::core::blueprint::{Definition, FieldDefinition, InputFieldDefinition};

///
/// A read optimized index for the blueprint.
pub struct BlueprintIndex {
    map: HashMap<String, (Definition, HashMap<String, FieldDef>)>,
    schema: SchemaDefinition,
}

#[derive(Debug)]
pub enum FieldDef {
    Field((FieldDefinition, HashMap<String, InputFieldDefinition>)),
    InputField(InputFieldDefinition),
}

impl FieldDef {
    pub fn get_arg(&self, arg_name: &str) -> Option<&InputFieldDefinition> {
        match self {
            FieldDef::Field((_, args)) => {
                // FIXME: use a hashmap to store the field args
                args.get(arg_name)
            }
            FieldDef::InputField(_) => None,
        }
    }
}

impl BlueprintIndex {
    pub fn init(blueprint: &Blueprint) -> Self {
        let mut map = HashMap::new();

        for definition in blueprint.definitions.iter() {
            match definition {
                Definition::Object(object_def) => {
                    let type_name = object_def.name.clone();
                    let mut fields_map = HashMap::new();

                    for field in &object_def.fields {
                        let args_map = HashMap::from_iter(
                            field
                                .args
                                .iter()
                                .map(|v| (v.name.clone(), v.clone()))
                                .collect::<Vec<_>>(),
                        );
                        fields_map.insert(
                            field.name.clone(),
                            FieldDef::Field((field.clone(), args_map)),
                        );
                    }

                    map.insert(
                        type_name,
                        (Definition::Object(object_def.to_owned()), fields_map),
                    );
                }
                Definition::Interface(interface_def) => {
                    let type_name = interface_def.name.clone();
                    let mut fields_map = HashMap::new();

                    for field in interface_def.fields.clone() {
                        let args_map = HashMap::from_iter(
                            field
                                .args
                                .iter()
                                .map(|v| (v.name.clone(), v.clone()))
                                .collect::<Vec<_>>(),
                        );
                        fields_map.insert(field.name.clone(), FieldDef::Field((field, args_map)));
                    }

                    map.insert(
                        type_name,
                        (Definition::Interface(interface_def.to_owned()), fields_map),
                    );
                }
                Definition::InputObject(input_object_def) => {
                    let type_name = input_object_def.name.clone();
                    let mut fields_map = HashMap::new();

                    for field in input_object_def.fields.clone() {
                        fields_map.insert(field.name.clone(), FieldDef::InputField(field));
                    }

                    map.insert(
                        type_name,
                        (
                            Definition::InputObject(input_object_def.to_owned()),
                            fields_map,
                        ),
                    );
                }
                Definition::Scalar(scalar_def) => {
                    let type_name = scalar_def.name.clone();
                    map.insert(
                        type_name.clone(),
                        (Definition::Scalar(scalar_def.to_owned()), HashMap::new()),
                    );
                }
                Definition::Enum(enum_def) => {
                    let type_name = enum_def.name.clone();
                    map.insert(
                        type_name.clone(),
                        (Definition::Enum(enum_def.to_owned()), HashMap::new()),
                    );
                }
                Definition::Union(union_def) => {
                    let type_name = union_def.name.clone();
                    map.insert(
                        type_name.clone(),
                        (Definition::Union(union_def.to_owned()), HashMap::new()),
                    );
                }
            }
        }

        Self { map, schema: blueprint.schema.to_owned() }
    }

    pub fn get_type(&self, type_name: &str) -> Option<&Definition> {
        self.map.get(type_name).map(|(definition, _)| definition)
    }

    pub fn get_field(&self, type_name: &str, field_name: &str) -> Option<&FieldDef> {
        self.map
            .get(type_name)
            .and_then(|(_, fields_map)| fields_map.get(field_name))
    }

    pub fn get_query_type(&self) -> Option<&Definition> {
        self.get_type(&self.schema.query)
    }

    pub fn get_query(&self) -> &String {
        &self.schema.query
    }

    pub fn get_mutation_type(&self) -> Option<&Definition> {
        self.schema.mutation.as_ref().and_then(|a| self.get_type(a))
    }

    pub fn get_mutation_type_name(&self) -> Option<&String> {
        self.schema.mutation.as_ref()
    }
}
