use std::collections::HashMap;

use tailcall_valid::Valid;

use crate::core::blueprint::*;
use crate::core::config;
use crate::core::config::Field;
use crate::core::ir::model::{Map, IR};
use crate::core::try_fold::TryFold;

pub fn update_enum_alias<'a>() -> TryFold<
    'a,
    (&'a ConfigModule, &'a Field, &'a config::Type, &'a str),
    FieldDefinition,
    BlueprintError,
> {
    TryFold::<(&ConfigModule, &Field, &config::Type, &'a str), FieldDefinition, BlueprintError>::new(
        |(config, field, _, _), mut b_field| {
            let enum_type = config.enums.get(field.type_of.name());
            if let Some(enum_type) = enum_type {
                let has_alias = enum_type.variants.iter().any(|v| v.alias.is_some());
                if !has_alias {
                    return Valid::succeed(b_field);
                }
                let mut map = HashMap::<String, String>::new();
                for v in enum_type.variants.iter() {
                    map.insert(v.name.clone(), v.name.clone());
                    if let Some(alias) = &v.alias {
                        for option in &alias.options {
                            map.insert(option.to_owned(), v.name.clone());
                        }
                    }
                }
                b_field.resolver = b_field
                    .resolver
                    .map(|r| IR::Map(Map { input: Box::new(r), map }));
            }
            Valid::succeed(b_field)
        },
    )
}
