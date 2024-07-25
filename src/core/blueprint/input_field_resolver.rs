use std::collections::HashMap;

use crate::core::blueprint::FieldDefinition;
use crate::core::config;
use crate::core::config::{ConfigModule, Field};
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
                b_field.args = b_field
                    .args
                    .into_iter()
                    .map(|mut arg| {
                        let of_type = &arg.of_type;
                        let mut renames: HashMap<Vec<String>, String> = HashMap::new();
                        let mut visited: Vec<String> = Vec::new();

                        fn extract_rename_paths(
                            path_and_target: (Vec<String>, &str),
                            config: &&ConfigModule,
                            renames: &mut HashMap<Vec<String>, String>,
                            visited: &mut Vec<String>,
                        ) {
                            let (path, target_name) = path_and_target;
                            if visited.contains(&target_name.to_string()) {
                                return;
                            }
                            visited.push(target_name.to_string());
                            for (type_name, metadata) in &config.types {
                                if target_name.eq(type_name) {
                                    for (field_name, field) in &metadata.fields {
                                        if let Some(modify) = &field.modify {
                                            if let Some(modified_name) = &modify.name {
                                                let mut new_path = path.clone();
                                                new_path.push(modified_name.to_string());
                                                renames.insert(new_path, field_name.to_string());
                                            }
                                        }
                                        let mut new_path = path.clone();
                                        new_path.push(field_name.to_string());
                                        extract_rename_paths(
                                            (new_path, &field.type_of),
                                            config,
                                            renames,
                                            visited,
                                        );
                                    }
                                }
                            }
                        }
                        extract_rename_paths(
                            (vec![], of_type.name()),
                            config,
                            &mut renames,
                            &mut visited,
                        );
                        arg.renames = renames;
                        arg
                    })
                    .collect::<Vec<_>>();
            }
            Valid::succeed(b_field)
        },
    )
}
