use std::collections::HashMap;

use crate::core::blueprint::FieldDefinition;
use crate::core::config;
use crate::core::config::{ConfigModule, Field};
use crate::core::ir::model::IR;
use crate::core::json::{JsonLike, JsonObjectLike};
use crate::core::try_fold::TryFold;
use crate::core::valid::Valid;

///
/// Our aim here is to construct the IR that modifies input arguments
/// - rename fields
/// - protect fields (TODO)
/// - sanitize input before sending (TODO)
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
                        // holds (type name) => Lens
                        let mut type_lenses: HashMap<String, InputTypeLens> = HashMap::new();

                        // used to keep the names of visited types so we don't visit them twice
                        let mut visited: Vec<String> = Vec::new();

                        // step: we extract the data required for the InputTransformsContext
                        extract_type_lenses(
                            input_type.name(),
                            config,
                            &mut visited,
                            &mut type_lenses,
                        );

                        // step: we optimized the produced lenses (remove empty ones)
                        let type_lenses = optimize_type_lenses(type_lenses);

                        let input_transforms = InputTransformsContext {
                            type_lenses,
                            arg_name: arg.name.clone(),
                            arg_type: arg.of_type.name().to_string(),
                        };

                        // step: return the resolver only if we have transforms to apply
                        if input_transforms.type_lenses.is_empty() {
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

///
/// Helper function that is used to recursively extract the lenses
fn extract_type_lenses(
    target_type: &str,
    config: &&ConfigModule,
    visited: &mut Vec<String>,
    type_lenses: &mut HashMap<String, InputTypeLens>,
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
        for (original_field_name, field) in &metadata.fields {
            let (mut field_name, field_type) =
                (original_field_name.to_string(), field.type_of.to_string());

            // step: we create a rename lens if it is applicable
            let rename_lens = if let Some(modify) = &field.modify {
                if let Some(modified_name) = &modify.name {
                    field_name = modified_name.to_string();
                    Some(InputTypeLens::Transform(
                        field_name.clone(),
                        InputFieldTransform::rename(original_field_name.clone()),
                    ))
                } else {
                    None
                }
            } else {
                None
            };

            // this lens is used to recursively apply lenses
            let type_lens = InputTypeLens::Transform(
                field_name.clone(),
                InputFieldTransform::field_type(field_type.clone()),
            );

            let local_lens = match rename_lens {
                Some(rename_lens) => InputTypeLens::compose(type_lens, rename_lens),
                None => type_lens,
            };

            // step: we compose the lens with the rest
            let lens = match type_lenses.remove(&target_type.to_string()) {
                Some(lens) => InputTypeLens::compose(lens, local_lens),
                None => local_lens,
            };

            // step: we put the lens back
            type_lenses.insert(target_type.to_string(), lens);

            // step: we try to extract the nested type lens
            extract_type_lenses(&field.type_of, config, visited, type_lenses);
        }
    }
}

///
/// Helper function that is used to remove empty lenses
fn optimize_type_lenses(
    type_lenses: HashMap<String, InputTypeLens>,
) -> HashMap<String, InputTypeLens> {
    type_lenses
        .clone()
        .into_iter()
        .filter_map(|(type_name, lens)| {
            if lens.is_empty() {
                None
            } else {
                Some((type_name, lens))
            }
        })
        .collect()
}

/// Used to contain all the directives that can apply on input type fields
#[derive(Clone, Debug)]
pub enum InputFieldTransform {
    /// Used to rename the field_name
    Rename(String),
    /// Used to contain the field_type so we can apply deeply nested lenses
    FieldType(String),
    // TODO: add more operations as time goes on
}

impl InputFieldTransform {
    fn rename(field_name: String) -> Self {
        Self::Rename(field_name)
    }

    fn field_type(field_type: String) -> Self {
        Self::FieldType(field_type)
    }
}

#[derive(Clone, Debug)]
/// Used to reconstruct input type objects
pub enum InputTypeLens {
    /// Used to compose two lenses together
    Compose(Box<InputTypeLens>, Box<InputTypeLens>),
    /// Used to apply a transformation to the lens
    Transform(String, InputFieldTransform),
}

impl InputTypeLens {
    /// Used to apply a set of lenses to the json value
    pub fn transform<'json, J>(
        &'json self,
        type_lenses: &'json HashMap<String, InputTypeLens>,
        value: &'json J,
    ) -> J
    where
        J: JsonLike<'json>,
    {
        if let Some(items) = value.as_array() {
            // if: it is an array, we iterate each item and we call recursively the
            // `transform` to apply the lens for each item.
            let arr = items
                .iter()
                .clone()
                .map(|item| self.transform(type_lenses, item))
                .collect::<Vec<_>>();
            J::array(arr)
        } else if let Some(obj) = value.as_object() {
            // if: it is an object, we apply the set of lenses to the object
            let mut new_map = J::JsonObject::new();

            self.recursive_prepare_object::<J>(type_lenses, obj, &mut new_map);

            J::object(new_map)
        } else {
            // if: anything else we just return it
            value.clone()
        }
    }

    ///
    /// Helper function that is used to apply a set of lenses to the object
    fn recursive_prepare_object<'json, J>(
        &'json self,
        type_lenses: &'json HashMap<String, InputTypeLens>,
        obj: &'json <J as JsonLike<'json>>::JsonObject<'json>,
        new_map: &mut <J as JsonLike<'json>>::JsonObject<'json>,
    ) where
        J: JsonLike<'json>,
    {
        match self {
            InputTypeLens::Compose(first, second) => {
                first.recursive_prepare_object::<J>(type_lenses, obj, new_map);
                second.recursive_prepare_object::<J>(type_lenses, obj, new_map);
            }
            InputTypeLens::Transform(path, operation) => match operation {
                InputFieldTransform::Rename(new_name) => {
                    if let Some(value) = new_map.remove_key(path) {
                        new_map.insert_key(new_name, value);
                    }
                }
                InputFieldTransform::FieldType(field_type) => {
                    // if: it is a field type we apply recursively the function to apply the
                    // appropriate lenses to the nested fields
                    if let Some(value) = obj.get_key(path) {
                        match type_lenses.get(field_type) {
                            Some(next_lens) => {
                                let value: <<J as JsonLike<'json>>::JsonObject<'json> as JsonObjectLike>::Value
                                = next_lens
                                    .transform::<<J::JsonObject<'json> as JsonObjectLike>::Value>(
                                        type_lenses,
                                        value,
                                    );
                                new_map.insert_key(path, value);
                            }
                            None => {
                                new_map.insert_key(path, value.clone());
                            }
                        }
                    }
                }
            },
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            InputTypeLens::Compose(first, second) => first.is_empty() && second.is_empty(),
            InputTypeLens::Transform(_, InputFieldTransform::Rename(_)) => false,
            InputTypeLens::Transform(_, InputFieldTransform::FieldType(_)) => true,
        }
    }

    fn compose(first: Self, second: Self) -> Self {
        Self::Compose(Box::new(first), Box::new(second))
    }
}

///
/// Used to hold input field transformations context
#[derive(Clone, Debug)]
pub struct InputTransformsContext {
    pub type_lenses: HashMap<String, InputTypeLens>,
    pub arg_name: String,
    pub arg_type: String,
}
