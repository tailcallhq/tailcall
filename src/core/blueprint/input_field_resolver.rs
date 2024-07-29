use std::collections::HashMap;

use crate::core::blueprint::FieldDefinition;
use crate::core::config;
use crate::core::config::{ConfigModule, Field};
use crate::core::ir::model::{InputTransforms, IR};
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
            if !field.args.is_empty() {
                let resolver = b_field
                    .args
                    .iter()
                    .filter_map(|arg| {
                        type InputFieldHashMap = HashMap<(String, String), String>;
                        let of_type = &arg.of_type;
                        let mut subfield_renames: InputFieldHashMap = HashMap::new();
                        let mut subfield_types: InputFieldHashMap = HashMap::new();
                        let mut visited: Vec<String> = Vec::new();

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
                            if visited.contains(&target_type.to_string()) {
                                return;
                            }
                            visited.push(target_type.to_string());
                            for (type_name, metadata) in &config.types {
                                if target_type.eq(type_name) {
                                    for (field_name, field) in &metadata.fields {
                                        let key = if let Some(modify) = &field.modify {
                                            if let Some(modified_name) = &modify.name {
                                                let key = (
                                                    target_type.to_string(),
                                                    modified_name.to_string(),
                                                );
                                                subfield_renames
                                                    .insert(key.clone(), field_name.to_string());
                                                key
                                            } else {
                                                (target_type.to_string(), field_name.to_string())
                                            }
                                        } else {
                                            (target_type.to_string(), field_name.to_string())
                                        };
                                        subfield_types.insert(key, field.type_of.clone());
                                        extract_rename_paths(
                                            &field.type_of,
                                            config,
                                            (subfield_renames, subfield_types, visited),
                                        );
                                    }
                                }
                            }
                        }
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

                        if !input_transforms.subfield_renames.is_empty() {
                            Some(IR::ModifyInput(input_transforms))
                        } else {
                            None
                        }
                    })
                    .reduce(|first, second| first.pipe(second));
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
