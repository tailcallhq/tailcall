use crate::core::blueprint::FieldDefinition;
use crate::core::config;
use crate::core::config::{ConfigModule, Field};
use crate::core::ir::model::IR;
use crate::core::ir::Discriminator;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, Validator};

fn compile_union_resolver(
    config: &ConfigModule,
    union_name: &str,
    union_: &config::Union,
) -> Valid<Discriminator, String> {
    Valid::from_iter(&union_.types, |type_name| {
        Valid::from_option(
            config
                .find_type(type_name)
                .map(|type_| (type_name.as_str(), type_)),
            "Can't find a type that is member of union type".to_string(),
        )
    })
    .and_then(|types| {
        let types: Vec<_> = types.into_iter().collect();

        Discriminator::new(union_name, &types)
    })
}

pub fn update_union_resolver<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, String>::new(
        |(config, field, _, _), mut b_field| {
            let Some(union_) = config.find_union(&field.type_of) else {
                return Valid::succeed(b_field);
            };

            compile_union_resolver(config, &field.type_of, union_).map(|discriminator| {
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
