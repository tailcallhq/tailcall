use std::collections::HashMap;

use crate::core::blueprint::{
    Blueprint, Definition, FieldDefinition, InputFieldDefinition, SchemaDefinition,
};

///
/// A read optimized index of all the fields in the Blueprint. Provide O(1)
/// access to getting any field information.
pub struct FieldIndex {
    map: HashMap<String, (Definition, HashMap<String, QueryField>)>,
    schema: SchemaDefinition,
}

#[derive(Debug)]
pub enum QueryField {
    Field((FieldDefinition, HashMap<String, InputFieldDefinition>)),
    InputField(InputFieldDefinition),
}

impl QueryField {
    pub fn get_arg(&self, arg_name: &str) -> Option<&InputFieldDefinition> {
        match self {
            QueryField::Field((_, args)) => args.get(arg_name),
            QueryField::InputField(_) => None,
        }
    }
}

impl FieldIndex {
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
                            QueryField::Field((field.clone(), args_map)),
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
                        fields_map.insert(field.name.clone(), QueryField::Field((field, args_map)));
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
                        fields_map.insert(field.name.clone(), QueryField::InputField(field));
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

    pub fn get_field(&self, type_name: &str, field_name: &str) -> Option<&QueryField> {
        self.map
            .get(type_name)
            .and_then(|(_, fields_map)| fields_map.get(field_name))
    }

    pub fn get_query(&self) -> &String {
        &self.schema.query
    }

    #[allow(unused)]
    pub fn get_mutation(&self) -> Option<&String> {
        self.schema.mutation.as_ref()
    }
}
