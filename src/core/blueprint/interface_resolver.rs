use std::collections::HashSet;

use crate::core::blueprint::FieldDefinition;
use crate::core::config;
use crate::core::config::{ConfigModule, Field};
use crate::core::ir::model::IR;
use crate::core::ir::Discriminator;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, Validator};

fn compile_interface_resolver(
    config: &ConfigModule,
    interface_name: &str,
    interface_types: HashSet<String>,
) -> Valid<Discriminator, String> {
    Valid::from_iter(&interface_types, |type_name| {
        Valid::from_option(
            config
                .find_type(type_name)
                .map(|type_| (type_name.as_str(), type_)),
            "Can't find a type that is member of interface type".to_string(),
        )
    })
    .and_then(|types| {
        let types: Vec<_> = types.into_iter().collect();

        Discriminator::new(interface_name, &types, crate::core::ir::DiscriminatorMode::Probability)
    })
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

            compile_interface_resolver(
                config,
                field.type_of.name(),
                interface_types.iter().cloned().collect(),
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
