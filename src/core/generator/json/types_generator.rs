use serde_json::{Map, Value};
use tailcall_valid::Valid;

use super::OperationTypeGenerator;
use crate::core::config::{Config, Field, Type};
use crate::core::generator::{NameGenerator, RequestSample};
use crate::core::helpers::gql_type::{is_primitive, is_valid_field_name, to_gql_type};
use crate::core::scalar::Scalar;
use crate::core::transform::Transform;

struct JSONValidator;

impl JSONValidator {
    /// checks if given json value is graphql compatible or not.
    fn is_graphql_compatible(value: &Value) -> bool {
        match value {
            Value::Array(json_array) => !json_array.is_empty(),
            Value::Object(json_object) => {
                !json_object.is_empty()
                    && !json_object
                        .keys()
                        .any(|json_property| !is_valid_field_name(json_property))
            }
            _ => true,
        }
    }
}

struct TypeMerger;

impl TypeMerger {
    /// given a list of types, merges all fields into single type.
    fn merge_fields(type_list: Vec<Type>) -> Type {
        let mut ty = Type::default();

        for current_type in type_list {
            for (key, new_field) in current_type.fields {
                if let Some(existing_field) = ty.fields.get(&key) {
                    if existing_field.type_of.name().is_empty()
                        || existing_field.type_of.name() == &Scalar::Empty.to_string()
                        || (existing_field.type_of.name() == &Scalar::JSON.to_string()
                            && new_field.type_of.name() != &Scalar::Empty.to_string())
                    {
                        ty.fields.insert(key, new_field);
                    }
                } else {
                    ty.fields.insert(key, new_field);
                }
            }
        }
        ty
    }
}

pub struct TypeGenerator<'a> {
    type_name_generator: &'a NameGenerator,
}

impl<'a> TypeGenerator<'a> {
    pub fn new(type_name_generator: &'a NameGenerator) -> Self {
        Self { type_name_generator }
    }

    fn generate_scalar(&self, config: &mut Config) -> Scalar {
        let any_scalar = Scalar::JSON;
        if config.types.contains_key(&any_scalar.name()) {
            return any_scalar;
        }
        any_scalar
    }

    fn create_type_from_object(
        &self,
        json_object: &'a Map<String, Value>,
        config: &mut Config,
    ) -> Type {
        let mut ty = Type::default();
        for (json_property, json_val) in json_object {
            let mut field = if !JSONValidator::is_graphql_compatible(json_val) {
                // if object, array is empty or object has in-compatible fields then
                // generate scalar for it.
                Field {
                    type_of: self.generate_scalar(config).to_string().into(),
                    ..Default::default()
                }
            } else {
                let mut field = Field::default();
                if is_primitive(json_val) {
                    field.type_of = to_gql_type(json_val).into();
                } else {
                    let type_name = self.generate_types(json_val, config);
                    field.type_of = type_name.into();
                }
                field
            };
            field.type_of = if json_val.is_array() {
                field.type_of.into_list()
            } else {
                field.type_of
            };

            ty.fields.insert(json_property.to_string(), field);
        }
        ty
    }

    pub fn generate_types(&self, json_value: &'a Value, config: &mut Config) -> String {
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
                        if !JSONValidator::is_graphql_compatible(json_item) {
                            return self.generate_scalar(config).to_string();
                        }
                        object_types.push(self.create_type_from_object(json_obj, config));
                    } else {
                        return self.generate_types(json_item, config);
                    }
                }

                if !object_types.is_empty() {
                    // merge the generated types of list into single concrete type.
                    let merged_type = TypeMerger::merge_fields(object_types);
                    let generate_type_name = self.type_name_generator.next();
                    config
                        .types
                        .insert(generate_type_name.to_owned(), merged_type);
                    return generate_type_name;
                }

                // generate a scalar if array is empty.
                self.generate_scalar(config).to_string()
            }
            Value::Object(json_obj) => {
                if !JSONValidator::is_graphql_compatible(json_value) {
                    return self.generate_scalar(config).to_string();
                }
                let ty = self.create_type_from_object(json_obj, config);
                let generate_type_name = self.type_name_generator.next();
                config.types.insert(generate_type_name.to_owned(), ty);
                generate_type_name
            }
            other => to_gql_type(other),
        }
    }
}

pub struct GraphQLTypesGenerator<'a> {
    request_sample: &'a RequestSample,
    type_name_generator: &'a NameGenerator,
}

impl<'a> GraphQLTypesGenerator<'a> {
    pub fn new(request_sample: &'a RequestSample, type_name_generator: &'a NameGenerator) -> Self {
        Self { request_sample, type_name_generator }
    }
}

impl Transform for GraphQLTypesGenerator<'_> {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        // generate the required types.
        let root_type = TypeGenerator::new(self.type_name_generator)
            .generate_types(&self.request_sample.res_body, &mut config);

        // generate the required field in operation type.
        OperationTypeGenerator.generate(
            self.request_sample,
            &root_type,
            self.type_name_generator,
            config,
        )
    }
}
