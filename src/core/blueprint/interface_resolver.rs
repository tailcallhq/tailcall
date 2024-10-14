use std::collections::BTreeSet;

use crate::core::blueprint::FieldDefinition;
use crate::core::config::{ConfigModule, Discriminate, Field, Type};
use crate::core::ir::model::IR;
use crate::core::ir::Discriminator;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, Validator};

fn compile_interface_resolver(
    interface_name: &str,
    interface_types: &BTreeSet<String>,
    discriminate: &Option<Discriminate>,
) -> Valid<Discriminator, String> {
    let typename_field = discriminate.as_ref().map(|d| d.field.clone());

    let mut types: Vec<_> = interface_types.clone().into_iter().collect();

    types.sort();

    Discriminator::new(
        interface_name.to_string(),
        types.into_iter().collect(),
        typename_field,
    )
}

pub fn update_interface_resolver<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a Type, &'a str), FieldDefinition, String> {
    TryFold::<(&ConfigModule, &Field, &Type, &str), FieldDefinition, String>::new(
        |(config, field, _, _), mut b_field| {
            let Some(interface_types) = config.interfaces_types_map().get(field.type_of.name())
            else {
                return Valid::succeed(b_field);
            };

            compile_interface_resolver(field.type_of.name(), interface_types, &field.discriminate)
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
