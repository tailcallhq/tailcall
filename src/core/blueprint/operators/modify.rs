use tailcall_valid::Valid;

use crate::core::blueprint::*;
use crate::core::config;
use crate::core::config::Field;
use crate::core::ir::model::IR;
use crate::core::try_fold::TryFold;

pub fn update_modify<'a>() -> TryFold<
    'a,
    (&'a ConfigModule, &'a Field, &'a config::Type, &'a str),
    FieldDefinition,
    BlueprintError,
> {
    TryFold::<(&ConfigModule, &Field, &config::Type, &'a str), FieldDefinition, BlueprintError>::new(
        |(config, field, type_of, _), mut b_field| {
            if let Some(modify) = field.modify.as_ref() {
                if let Some(new_name) = &modify.name {
                    for name in type_of.implements.iter() {
                        let interface = config.find_type(name);
                        if let Some(interface) = interface {
                            if interface.fields.iter().any(|(name, _)| name == new_name) {
                                return Valid::fail(BlueprintError::FieldExistsInInterface);
                            }
                        }
                    }
                    b_field.resolver = Some(
                        b_field
                            .resolver
                            .unwrap_or(IR::ContextPath(vec![b_field.name.clone()])),
                    );
                    b_field = b_field.name(new_name.clone());
                }
            }
            Valid::succeed(b_field)
        },
    )
}
