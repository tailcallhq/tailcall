use std::collections::BTreeSet;

use crate::core::blueprint::FieldDefinition;
use crate::core::config;
use crate::core::config::{ConfigModule, Field};
use crate::core::ir::model::IR;
use crate::core::ir::Discriminator;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, Validator};

fn compile_interface_resolver(
    interface_name: &str,
    interface_types: BTreeSet<String>,
    interface_type: &config::Type,
) -> Valid<Discriminator, String> {
    let typename_field = interface_type
        .discriminate
        .as_ref()
        .map(|d| d.field.clone());

    Discriminator::new(interface_name.to_string(), interface_types, typename_field)
}

pub fn update_interface_resolver<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, String>::new(
        |(config, field, _, _), mut b_field| {
            let Some(interface_types) = config.interfaces_types_map().get(field.type_of.name())
            else {
                return Valid::succeed(b_field);
            };

            let Some(interface_type) = config.find_type(field.type_of.name()) else {
                return Valid::succeed(b_field);
            };

            compile_interface_resolver(
                field.type_of.name(),
                interface_types.clone(),
                interface_type,
            )
            .map(|discriminator| {
                b_field.resolver = Some(
                    b_field
                        .resolver
                        .unwrap_or(IR::ContextPath(vec![b_field.name.clone()])),
                );
                b_field.map_expr(move |expr| IR::Discriminate(discriminator, expr.into()));
                b_field
            })
        },
    )
}
