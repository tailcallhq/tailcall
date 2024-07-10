use std::collections::{BTreeMap, BTreeSet};

use inflector::Inflector;

use crate::core::config::Config;
use crate::core::valid::{Valid, Validator};
use crate::core::Transform;

/// **Case styles**
/// - Field names should use `camelCase`.
/// - Type names should use `PascalCase`.
/// - Enum names should use `PascalCase`.
/// - Enum values should use `ALL_CAPS`, because they are similar to constants.
#[derive(Default)]
pub struct Linter;

impl Transform for Linter {
    type Value = Config;
    type Error = String;

    fn transform(&self, value: Self::Value) -> Valid<Self::Value, Self::Error> {
        Valid::succeed(value)
            .and_then(resolve_types_and_fields)
            .and_then(resolve_enum)
    }
}

fn resolve_types_and_fields(mut config: Config) -> Valid<Config, String> {
    // Handle Types
    let mut resolved_types = BTreeMap::new();
    for (mut type_name, mut type_) in config.types {
        // Handle Fields
        let mut resolved_fields = BTreeMap::new();
        for (mut field_name, mut field) in type_.fields {
            field.type_of = field.type_of.to_pascal_case();

            // Update type names in arg
            for (_, arg) in field.args.iter_mut() {
                arg.type_of = arg.type_of.to_pascal_case();
            }
            field_name = field_name.to_camel_case();
            resolved_fields.insert(field_name, field);
        }

        // Insert resolved fields
        type_.fields = resolved_fields;

        // Insert resolved types
        type_name = type_name.to_pascal_case();
        resolved_types.insert(type_name, type_);
    }
    // Insert resolved types
    config.types = resolved_types;

    Valid::succeed(config)
}

fn resolve_enum(mut config: Config) -> Valid<Config, String> {
    let mut resolved_enums = BTreeMap::new();
    // Handle Enums and Enum Values
    for (mut enum_name, mut enum_) in config.enums {
        let mut resolved_vals = BTreeSet::new();

        for mut enum_val in enum_.variants {
            enum_val.name = enum_val.name.to_uppercase();
            resolved_vals.insert(enum_val);
        }
        enum_.variants = resolved_vals;

        enum_name = enum_name.to_pascal_case();
        resolved_enums.insert(enum_name, enum_);
    }
    config.enums = resolved_enums;

    Valid::succeed(config)
}

#[cfg(test)]
mod tests {
    use crate::core::config::*;
    use crate::core::valid::Validator;
    use crate::core::Transform;

    #[test]
    fn test_linter() {
        let config = Config::from_sdl(
            &std::fs::read_to_string(tailcall_fixtures::configs::LINT_ERRORS).unwrap(),
        )
        .to_result()
        .unwrap();
        let linter = super::Linter;
        let result = linter.transform(config).to_result().unwrap();
        insta::assert_snapshot!(result.to_sdl());
    }
}
