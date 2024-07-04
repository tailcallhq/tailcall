use std::collections::BTreeMap;

use crate::core::config::{Config, Field, Type};
use crate::core::transform::{self, Transform};
use crate::core::valid::Valid;

/// Flatten single types
/// If a type has only one field, it is considered a single type.
/// If a field references a single type, it will be flattened.
#[derive(Default)]
pub struct FlattenSingleTypes;

impl Transform for FlattenSingleTypes {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Config) -> Valid<Config, String> {
        let single_types = get_single_types(&config);
        let root_types = get_root_types(&config);
        let mut still_referenced_single_types = Vec::<String>::new();

        config.types.iter_mut().for_each(|(_, ty)| {
            let fields_referencing_single_type = ty
                .fields
                .iter_mut()
                .filter(|(_, field)| single_types.contains_key(&field.type_of));

            // Skip fields with resolver and prevent array of array
            let (skipped_fields, fields_to_flatten): (Vec<_>, Vec<_>) =
                fields_referencing_single_type.partition(|(_, field)| {
                    field.has_resolver() || (field.list && single_types[&field.type_of].list)
                });

            // Keep track of single types that are still referenced
            still_referenced_single_types.extend(
                skipped_fields
                    .iter()
                    .map(|(_, field)| field.type_of.clone()),
            );

            fields_to_flatten.into_iter().for_each(|(_, field)| {
                *field = flatten_field(field.clone(), &single_types);
            });
        });

        // Remove all single types, take care of keeping root types even if it's a single type
        // Also keep single types that are still referenced
        config.types.retain(|type_name, ty| {
            root_types.contains(type_name)
                || !is_single_type(ty)
                || still_referenced_single_types.contains(type_name)
        });

        transform::default().transform(config)
    }
}

fn get_root_types(config: &Config) -> Vec<String> {
    vec![
        config.schema.query.clone().unwrap_or("Query".to_string()),
        config
            .schema
            .mutation
            .clone()
            .unwrap_or("Mutation".to_string()),
        config
            .schema
            .subscription
            .clone()
            .unwrap_or("Subscription".to_string()),
    ]
}

/// Collect all single types and their unique field type
fn get_single_types(config: &Config) -> BTreeMap<String, Field> {
    config
        .types
        .iter()
        .filter(|(_, ty)| is_single_type(ty))
        .map(|(name, ty)| (name.clone(), ty.fields.values().next().unwrap().clone()))
        .collect()
}

/// Recursively resolve single types
fn flatten_field(field: Field, single_types: &BTreeMap<String, Field>) -> Field {
    if let Some(single_type_field) = single_types.get(&field.type_of) {
        let mut new_field = single_type_field.clone();

        // If one of the merged field is a list, the new type will be a list
        new_field.list = single_type_field.list || field.list;

        // If all the merged fields are required, the new type will be required
        // There is special case for list, we prevented array of array in the transform
        if single_type_field.list {
            new_field.required = field.required && single_type_field.required;
            new_field.list_type_required = single_type_field.list_type_required;
        } else if field.list {
            new_field.required = field.required;
            new_field.list_type_required = field.list_type_required && single_type_field.required;
        } else {
            new_field.required = field.required && single_type_field.required;
        }

        flatten_field(new_field, single_types)
    } else {
        field
    }
}

/// Check if type is single type
fn is_single_type(ty: &Type) -> bool {
    let not_omitted_fields_count = ty
        .fields
        .iter()
        .filter(|(_, field)| field.omit.is_none())
        .count();
    let added_fields_count = ty.added_fields.len();
    let total_fields_count = not_omitted_fields_count + added_fields_count;

    // Flattened types should not have any tag
    ty.tag.is_none() && total_fields_count == 1
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use std::fs;
    use tailcall_fixtures::configs;

    use super::FlattenSingleTypes;
    use crate::core::config::Config;
    use crate::core::transform::Transform;
    use crate::core::valid::Validator;

    #[test]
    fn test_flatten_single_types_transform() {
        let config = Config::from_sdl(
            fs::read_to_string(configs::FLATTEN_SINGLE_TYPES_CONFIG)
                .unwrap()
                .as_str(),
        )
        .to_result()
        .unwrap();

        let transformed_config = FlattenSingleTypes.transform(config).to_result().unwrap();
        assert_snapshot!(transformed_config.to_sdl());
    }

    #[test]
    fn test_flatten_single_types_with_array_transform() {
        let config = Config::from_sdl(
            fs::read_to_string(configs::FLATTEN_SINGLE_TYPES_WITH_ARRAY_CONFIG)
                .unwrap()
                .as_str(),
        )
        .to_result()
        .unwrap();

        let transformed_config = FlattenSingleTypes.transform(config).to_result().unwrap();
        assert_snapshot!(transformed_config.to_sdl());
    }

    #[test]
    fn test_flatten_single_types_with_resolver_transform() {
        let config = Config::from_sdl(
            fs::read_to_string(configs::FLATTEN_SINGLE_TYPES_WITH_RESOLVER_CONFIG)
                .unwrap()
                .as_str(),
        )
        .to_result()
        .unwrap();

        let transformed_config = FlattenSingleTypes.transform(config).to_result().unwrap();
        assert_snapshot!(transformed_config.to_sdl());
    }
}
