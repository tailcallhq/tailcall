use serde_json::{Map, Value};

use super::NameGenerator;
use crate::core::config::transformer::Transform;
use crate::core::config::{Config, Field, Type};
use crate::core::helpers::gql_type::{is_primitive, is_valid_field_name, to_gql_type};
use crate::core::valid::Valid;

pub struct TypesGenerator<'a, 'b, T1: OperationGenerator, T2: NameGenerator> {
    json_value: &'a Value,
    operation_generator: T1,
    type_name_generator: &'b mut T2,
}

impl<'a, 'b, T1, T2> TypesGenerator<'a, 'b, T1, T2>
where
    T1: OperationGenerator,
    T2: NameGenerator,
{
    pub fn new(
        json_value: &'a Value,
        operation_generator: T1,
        type_name_generator: &'b mut T2,
    ) -> Self {
        Self { json_value, operation_generator, type_name_generator }
    }
}

impl<'a, T1, T2> TypesGenerator<'a, '_, T1, T2>
where
    T1: OperationGenerator,
    T2: NameGenerator,
{
    // checks if json value is compatible with graphql or not.
    fn should_generate_type(&self, value: &'a Value) -> bool {
        match value {
            Value::Array(json_array) => !json_array.is_empty(),
            Value::Object(json_object) => {
                !json_object.is_empty()
                    && !json_object
                        .keys()
                        .any(|json_property| !is_valid_field_name(json_property))
            }
            _ => true, // generate for all primitive types.
        }
    }

    fn generate_scalar(&self, config: &mut Config) -> String {
        let any_scalar = "Any";
        if config.types.contains_key(any_scalar) {
            return any_scalar.to_string();
        }
        config.types.insert(any_scalar.to_string(), Type::default());
        any_scalar.to_string()
    }

    fn create_type_from_object(
        &mut self,
        json_object: &'a Map<String, Value>,
        config: &mut Config,
    ) -> Type {
        let mut ty = Type::default();
        for (json_property, json_val) in json_object {
            let field = if !self.should_generate_type(json_val) {
                // if object, array is empty or object has in-compatible fields then
                // generate scalar for it.
                Field {
                    type_of: self.generate_scalar(config),
                    list: json_val.is_array(),
                    ..Default::default()
                }
            } else {
                let mut field = Field::default();
                if is_primitive(json_val) {
                    field.type_of = to_gql_type(json_val);
                } else {
                    let type_name = self.generate_types(json_val, config);
                    field.type_of = type_name;
                    field.list = json_val.is_array()
                }
                field
            };
            ty.fields.insert(json_property.to_string(), field);
        }
        ty
    }

    /// given a list of types, merges all fields into single type.
    fn merge_types(&self, type_list: Vec<Type>) -> Type {
        let mut ty = Type::default();
        for current_type in type_list {
            for (key, value) in current_type.fields {
                if let Some(existing_value) = ty.fields.get(&key) {
                    if existing_value.type_of.is_empty()
                        || existing_value.type_of == "Empty"
                        || existing_value.type_of == "Any"
                    {
                        ty.fields.insert(key, value);
                    }
                } else {
                    ty.fields.insert(key, value);
                }
            }
        }
        ty
    }

    fn generate_types(&mut self, json_value: &'a Value, config: &mut Config) -> String {
        match json_value {
            Value::Array(json_arr) => {
                let vec_capacity = json_arr.first().map_or(0, |json_item| {
                    if json_item.is_object() {
                        json_arr.len()
                    } else {
                        0
                    }
                });
                let mut object_types = Vec::<_>::with_capacity(vec_capacity);
                for json_item in json_arr {
                    if let Value::Object(json_obj) = json_item {
                        if !self.should_generate_type(json_item) {
                            return self.generate_scalar(config);
                        }
                        object_types.push(self.create_type_from_object(json_obj, config));
                    } else {
                        return self.generate_types(json_item, config);
                    }
                }

                if !object_types.is_empty() {
                    // merge the generated types of list into single concrete type.
                    let merged_type = self.merge_types(object_types);
                    let generate_type_name = self.type_name_generator.generate_name();
                    config.types.insert(generate_type_name.clone(), merged_type);
                    return generate_type_name;
                }

                // generate a scalar if array is empty.
                self.generate_scalar(config)
            }
            Value::Object(json_obj) => {
                if !self.should_generate_type(json_value) {
                    return self.generate_scalar(config);
                }
                let ty = self.create_type_from_object(json_obj, config);
                let generate_type_name = self.type_name_generator.generate_name();
                config.types.insert(generate_type_name.clone(), ty);
                generate_type_name
            }
            other => to_gql_type(other),
        }
    }
}

impl<T1, T2> Transform for TypesGenerator<'_, '_, T1, T2>
where
    T1: OperationGenerator,
    T2: NameGenerator,
{
    fn transform(&mut self, mut config: Config) -> Valid<Config, String> {
        let root_type_name = self.generate_types(self.json_value, &mut config);
        self.operation_generator
            .generate(root_type_name.as_str(), config)
    }
}

/// For generated types we also have to generate the appropriate operation type.
/// OperationGenerator should be implemented by Query, Subscription and
/// Mutation.
pub trait OperationGenerator {
    fn generate(&self, root_type: &str, config: Config) -> Valid<Config, String>;
}
