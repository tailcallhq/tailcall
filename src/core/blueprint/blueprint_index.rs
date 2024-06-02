use std::collections::HashMap;
use crate::core::blueprint::{Definition, FieldDefinition, InputFieldDefinition};

#[derive(Debug)]
pub struct BlueprintIndex {
    pub map: HashMap<String, (Definition, HashMap<String, FieldDef>)>
}

#[derive(Debug)]
pub enum FieldDef {
    Field(FieldDefinition),
    InputFieldDefinition(InputFieldDefinition)
}

impl BlueprintIndex {
    pub fn init(definitions: Vec<Definition>) -> Self {
        let mut map = HashMap::new();

        for definition in definitions {
            match definition {
                Definition::Object(object_def) => {
                    let type_name = object_def.name.clone();
                    let mut fields_map = HashMap::new();

                    for field in &object_def.fields {
                        fields_map.insert(field.name.clone(), FieldDef::Field(field.clone()));
                    }

                    map.insert(type_name, (Definition::Object(object_def), fields_map));
                }
                Definition::Interface(interface_def) => {
                    let type_name = interface_def.name.clone();
                    let mut fields_map = HashMap::new();

                    for field in interface_def.fields.clone() {
                        fields_map.insert(field.name.clone(), FieldDef::Field(field));
                    }

                    map.insert(type_name, (Definition::Interface(interface_def), fields_map));
                }
                Definition::InputObject(input_object_def) => {
                    let type_name = input_object_def.name.clone();
                    let mut fields_map = HashMap::new();

                    for field in input_object_def.fields.clone() {
                        fields_map.insert(field.name.clone(), FieldDef::InputFieldDefinition(field));
                    }

                    map.insert(type_name, (Definition::InputObject(input_object_def), fields_map));
                }
                Definition::Scalar(scalar_def) => {
                    let type_name = scalar_def.name.clone();
                    map.insert(type_name.clone(), (Definition::Scalar(scalar_def), HashMap::new()));
                }
                Definition::Enum(enum_def) => {
                    let type_name = enum_def.name.clone();
                    map.insert(type_name.clone(), (Definition::Enum(enum_def), HashMap::new()));
                }
                Definition::Union(union_def) => {
                    let type_name = union_def.name.clone();
                    map.insert(type_name.clone(), (Definition::Union(union_def), HashMap::new()));
                }
            }
        }

        Self {
            map
        }
    }

    pub fn get_type(&self, type_name: &str) -> Option<&Definition> {
        self.map.get(type_name).map(|(definition, _)| definition)
    }

    pub fn get_field(&self, query: &str, field_name: &str) -> Option<&FieldDef> {
        self.map.get(query).and_then(|(_, fields_map)| fields_map.get(field_name))
    }
}