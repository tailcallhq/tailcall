use std::collections::HashMap;

use crate::core::blueprint::FieldDefinition;
use crate::core::config;
use crate::core::config::{ConfigModule, Field};
use crate::core::ir::model::{InputTransforms, TransformKey, IR};
use crate::core::try_fold::TryFold;
use crate::core::valid::Valid;

///
/// Our aim here is to construct the IR that will perform the following
/// operations
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
                // step: construct a resolver (optional) for the operations we want to perform
                // iter: for every argument on the field we check if `@modify` directives are
                // applied on it
                let resolver = b_field
                    .args
                    .iter()
                    .filter_map(|arg| {
                        type InputFieldHashMap = HashMap<TransformKey, String>;
                        let of_type = &arg.of_type;
                        // holds (type name, field name) => new field name
                        // used to give a rename to a field if the
                        // (type name, field name) combination exists in the HashMap
                        let mut subfield_renames: InputFieldHashMap = HashMap::new();
                        // holds (type name, field name) => type name
                        // used to lookup the type of the field when we recursively
                        // try to rename the fields of complex structures
                        let mut subfield_types: InputFieldHashMap = HashMap::new();
                        // used to keep the names of visited types so we don't visit them twice
                        let mut visited: Vec<String> = Vec::new();

                        ///
                        /// Helper function that is used to recursively extract
                        /// the required data for the
                        /// input transform context.
                        fn extract_rename_paths(
                            target_type: &str,
                            config: &&ConfigModule,
                            data: (
                                &mut InputFieldHashMap,
                                &mut InputFieldHashMap,
                                &mut Vec<String>,
                            ),
                        ) {
                            let (subfield_renames, subfield_types, visited) = data;
                            // step: check if we visited the type to prevent infinite looping on
                            // recursive types
                            if visited.contains(&target_type.to_string()) {
                                return;
                            }
                            // step: we append the type name so we don't visit it again
                            visited.push(target_type.to_string());
                            // step: we find the metadata for the associated type
                            if let Some((_, metadata)) = config
                                .types
                                .iter()
                                .find(|(type_name, _)| type_name.as_str().eq(target_type)) {
                                // iter: for every field in the type
                                for (field_name, field) in &metadata.fields {
                                    let key = if let Some(modify) = &field.modify {
                                        // step: we collect the field rename if `@modify`
                                        // directive is applied
                                        if let Some(modified_name) = &modify.name {
                                            let key = TransformKey::from_str(
                                                target_type.to_string(),
                                                modified_name.to_string(),
                                            );
                                            subfield_renames
                                                .insert(key.clone(), field_name.to_string());
                                            key
                                        } else {
                                            TransformKey::from_str(
                                                target_type.to_string(),
                                                field_name.to_string(),
                                            )
                                        }
                                    } else {
                                        TransformKey::from_str(
                                            target_type.to_string(),
                                            field_name.to_string(),
                                        )
                                    };

                                    // step: we collect the field type
                                    subfield_types.insert(key, field.type_of.clone());

                                    // step: we go deeper in case the field implements an object
                                    // type
                                    extract_rename_paths(
                                        &field.type_of,
                                        config,
                                        (subfield_renames, subfield_types, visited),
                                    );
                                }
                            }
                        }

                        // step: call extract_rename_paths with the aim to populate the
                        // subfield_renames and subfield_types HashMaps
                        extract_rename_paths(
                            of_type.name(),
                            config,
                            (&mut subfield_renames, &mut subfield_types, &mut visited),
                        );

                        let input_transforms = InputTransforms {
                            subfield_renames,
                            subfield_types,
                            arg_name: arg.name.clone(),
                            arg_type: arg.of_type.name().to_string(),
                        };

                        // step: return the resolver only if we have renames to apply
                        // so we do not put extra overhead on the resolver if it is empty
                        if !input_transforms.subfield_renames.is_empty() {
                            Some(IR::ModifyInput(input_transforms))
                        } else {
                            None
                        }
                    })
                    .reduce(|first, second| first.pipe(second));

                // step: we ensure that combine our produce resolver with the existing one
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
