use std::collections::{BTreeMap, BTreeSet};

use convert_case::Casing;
use serde::{Deserialize, Serialize};

use crate::config::{AddField, Config, Type};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
pub struct Lint {
    #[serde(rename = "field")]
    pub field_lint: bool,
    #[serde(rename = "type")]
    pub type_lint: bool,
    #[serde(rename = "enum")]
    pub enum_lint: bool,
    #[serde(rename = "enumValue")]
    pub enum_value_lint: bool,
    #[serde(rename = "autoFix")]
    pub autofix: bool,
}

// Type -> PascalCase
// Field -> camelCase
// Enum -> PascalCase
// Enum Values -> ALL_CAPS

pub fn lint_fix(mut config: Config) -> Config {
    if let Some(lint) = &config.server.lint {
        let mut fixed_types = BTreeMap::new();
        for (k, ty) in config.types.iter() {
            fixed_types.insert(
                match lint.type_lint {
                    true => k.to_case(convert_case::Case::Pascal),
                    false => k.clone(),
                },
                fix_type(lint, ty),
            );
        }
        config.types = fixed_types;
    }
    config
}

// TODO: check variants, fields and added_fields
fn fix_type(lint: &Lint, ty: &Type) -> Type {
    let mut new_ty = Type::default();

    // for enum
    if lint.enum_lint {
        if let Some(variants) = &ty.variants {
            let mut set = BTreeSet::new();
            for variant in variants {
                set.insert(variant.to_case(convert_case::Case::Upper));
            }
            new_ty.variants = Some(set);
        }
    }

    if lint.field_lint {
        for (k, field) in &ty.fields {
            new_ty
                .fields
                .insert(k.to_case(convert_case::Case::Camel), field.clone());
        }
        for field in &ty.added_fields {
            new_ty.added_fields.push(AddField {
                name: field.name.to_case(convert_case::Case::Camel),
                path: field.path.clone(),
            })
        }
    }

    new_ty
}
