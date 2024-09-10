use std::collections::HashMap;

use async_graphql::Name;
use async_graphql_value::ConstValue;
use indexmap::IndexMap;

use crate::core::blueprint::FieldDefinition;
use crate::core::config;
use crate::core::config::{ConfigModule, Field};
use crate::core::ir::model::IR;
use crate::core::json::{JsonLike, Lens};
use crate::core::try_fold::TryFold;
use crate::core::valid::Valid;

/// Our aim here is to construct the IR that modifies input arguments
pub fn update_input_field_resolver<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, String>::new(
        |(config, field, _typ, _), mut b_field| {
            // step: we check that the field has arguments
            if !field.args.is_empty() {
                // step: we construct the resolver
                // iter: for every input field
                let resolver = b_field
                    .args
                    .iter()
                    .filter_map(|arg| {
                        let input_type = &arg.of_type;
                        // holds (type name) => Context
                        let mut types_context: HashMap<String, InputFieldContext> = HashMap::new();

                        // used to keep the names of visited types so we don't visit them twice
                        let mut visited: Vec<String> = Vec::new();

                        // step: we extract the data required for the InputTransformsContext
                        extract_types_context(
                            input_type.name(),
                            config,
                            &mut visited,
                            &mut types_context,
                        );

                        // step: we optimized the produced context (remove empty ones)
                        let types_context = optimize_types_context(types_context);

                        let input_transforms = InputTransformsContext {
                            types_context,
                            arg_name: arg.name.clone(),
                            arg_type: arg.of_type.name().to_string(),
                        };

                        // step: return the resolver only if we have transforms to apply
                        if input_transforms.types_context.is_empty() {
                            None
                        } else {
                            Some(IR::ModifyInput(input_transforms))
                        }
                    })
                    .reduce(|first, second| first.pipe(second));

                // step: we chain our produced resolver with the existing one
                b_field.resolver = match (b_field.resolver, resolver) {
                    (None, None) => None,
                    (None, Some(input_resolvers)) => Some(input_resolvers),
                    (Some(field_resolver), None) => Some(field_resolver),
                    (Some(field_resolver), Some(input_resolvers)) => {
                        Some(input_resolvers.pipe(field_resolver))
                    }
                };
            };

            Valid::succeed(b_field)
        },
    )
}

/// Helper function that is used to recursively extract the operations context
fn extract_types_context(
    target_type: &str,
    config: &&ConfigModule,
    visited: &mut Vec<String>,
    types_context: &mut HashMap<String, InputFieldContext>,
) {
    // step: if we visited the type we skip
    if visited.contains(&target_type.to_string()) {
        return;
    }

    // step: we append the type name so we don't visit it again
    visited.push(target_type.to_string());

    // step: we collect the metadata for the associated type
    // iter: for every field in the type
    if let Some((_, metadata)) = config
        .types
        .iter()
        .find(|(type_name, _)| type_name.as_str().eq(target_type))
    {
        let mut types: Vec<(Lens, String)> = Vec::new();
        let mut operations: Vec<(Lens, Operation)> = Vec::new();

        for (original_field_name, field) in &metadata.fields {
            let (mut field_name, field_type) = (
                original_field_name.to_string(),
                field.type_of.name().to_string(),
            );

            // step: we record the rename operation
            let operation_lens = if let Some(modify) = &field.modify {
                if let Some(modified_name) = &modify.name {
                    field_name = modified_name.to_string();
                    Some((
                        Lens::Select(field_name.clone()),
                        Operation::Rename(original_field_name.to_string()),
                    ))
                } else {
                    None
                }
            } else {
                None
            };

            // step: after we finish composing the operations we collect them
            if let Some(lens) = operation_lens {
                operations.push(lens);
            }

            // step: we collect the type of the field to aid the recursive parsing of the
            // object
            types.push((Lens::Select(field_name), field_type));

            // step: we go deeper to check it is nested type
            extract_types_context(field.type_of.name(), config, visited, types_context);
        }

        // step: putting the context together
        let input_field_context = InputFieldContext { types, operations };

        // step: we collect the context
        types_context.insert(target_type.to_string(), input_field_context);
    }
}

/// Helper function that is used to remove empty operations
fn optimize_types_context(
    types_context: HashMap<String, InputFieldContext>,
) -> HashMap<String, InputFieldContext> {
    let mut operations_count: HashMap<String, usize> = HashMap::new();

    // step: we collect the types and their operations count
    for (type_name, context) in types_context.iter() {
        operations_count.insert(type_name.to_string(), context.operations.len());
    }

    let mut new_types_context = HashMap::new();

    for (type_name, context) in types_context.into_iter() {
        let outer_count = operations_count.get(&type_name).unwrap_or(&0);
        if outer_count > &0 {
            let new_input_context = InputFieldContext {
                types: context
                    .types
                    .into_iter()
                    .filter(|(_field_name, type_name)| {
                        let inner_count = operations_count.get(type_name).unwrap_or(&0);
                        // step: keep only fields that points to other types that contain operations
                        inner_count > &0
                    })
                    .collect(),
                operations: context.operations,
            };

            new_types_context.insert(type_name, new_input_context);
        }
    }

    new_types_context
}

/// Used to contain the required context that allows to perform various
/// operations on input field types
#[derive(Clone, Debug)]
pub struct InputFieldContext {
    pub types: Vec<(Lens, String)>,
    pub operations: Vec<(Lens, Operation)>,
}

/// Used to contain all the operations you can apply on input field values
#[derive(Clone, Debug)]
pub enum Operation {
    /// Used to rename the field_name
    Rename(String),
    /// Used to chains one or more operations
    Compose(Box<Self>, Box<Self>),
}

/// Used to hold input field transformations context
#[derive(Clone, Debug)]
pub struct InputTransformsContext {
    pub types_context: HashMap<String, InputFieldContext>,
    pub arg_name: String,
    pub arg_type: String,
}

impl InputTransformsContext {
    pub fn transform(&self, args: &mut IndexMap<Name, ConstValue>) {
        let field_name = Name::new(self.arg_name.clone());
        if let Some(mut value) = args.swap_remove(&field_name) {
            self.recursive_transform(&self.arg_type, &mut value);
            args.insert(field_name, value);
        }
    }

    pub fn recursive_transform<Json>(&self, target_type: &str, value: &mut Json)
    where
        for<'json> Json: JsonLike<'json>,
    {
        if let Some(context) = self.types_context.get(target_type) {
            // step: iterate recursive types and calculate their value
            for (type_lens, type_name) in context.types.iter() {
                if let Some(mut local_value) = type_lens.remove(value) {
                    if let Some(_obj) = local_value.as_object_mut() {
                        self.recursive_transform(type_name, &mut local_value);
                    } else if let Some(arr) = local_value.as_array_mut() {
                        for item in arr.iter_mut() {
                            self.recursive_transform(type_name, item);
                        }
                    }
                    type_lens.set(value, local_value);
                }
            }

            // step: apply operations
            for (operation_lens, operation) in context.operations.iter() {
                if let Some(local_value) = operation_lens.remove(value) {
                    let (new_lens, updated_value) =
                        Self::recursive_operation(operation, operation_lens.clone(), local_value);
                    new_lens.set(value, updated_value);
                }
            }
        }
    }

    pub fn recursive_operation<Json>(
        operation: &Operation,
        _lens: Lens,
        value: Json,
    ) -> (Lens, Json) {
        match operation {
            Operation::Rename(new_name) => (Lens::select(new_name), value),
            Operation::Compose(first, second) => {
                let (lens, value) = Self::recursive_operation(first, _lens, value);
                let (lens, value) = Self::recursive_operation(second, lens, value);
                (lens, value)
            }
        }
    }
}
