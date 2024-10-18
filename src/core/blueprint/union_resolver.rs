use crate::core::blueprint::FieldDefinition;
use crate::core::config::{ConfigModule, Discriminate, Field, Type, Union};
use crate::core::ir::model::IR;
use crate::core::ir::Discriminator;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, Validator};

fn compile_union_resolver(
    union_name: &str,
    union_definition: &Union,
    discriminate: &Option<Discriminate>,
) -> Valid<Discriminator, String> {
    let typename_field = discriminate.as_ref().map(|d| d.get_field());

    Discriminator::new(
        union_name.to_string(),
        union_definition.types.clone(),
        typename_field,
    )
}

pub fn update_union_resolver<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a Type, &'a str), FieldDefinition, String> {
    TryFold::<(&ConfigModule, &Field, &Type, &str), FieldDefinition, String>::new(
        |(config, field, _, _), mut b_field| {
            let Some(union_definition) = config.find_union(field.type_of.name()) else {
                return Valid::succeed(b_field);
            };

            compile_union_resolver(field.type_of.name(), union_definition, &field.discriminate).map(
                |discriminator| {
                    b_field.resolver = Some(
                        b_field
                            .resolver
                            .unwrap_or(IR::ContextPath(vec![b_field.name.clone()])),
                    );
                    b_field.map_expr(move |expr| IR::Discriminate(discriminator, expr.into()));
                    b_field
                },
            )
        },
    )
}
