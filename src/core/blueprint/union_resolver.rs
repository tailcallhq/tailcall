use tailcall_valid::{Valid, Validator};

use super::BlueprintError;
use crate::core::blueprint::FieldDefinition;
use crate::core::config::{ConfigModule, Discriminate, Field, Type, Union};
use crate::core::ir::model::IR;
use crate::core::ir::Discriminator;
use crate::core::try_fold::TryFold;

fn compile_union_resolver(
    union_name: &str,
    union_definition: &Union,
    discriminate: &Option<Discriminate>,
) -> Valid<Discriminator, BlueprintError> {
    let typename_field = discriminate.as_ref().map(|d| d.get_field());

    match Discriminator::new(
        union_name.to_string(),
        union_definition.types.clone(),
        typename_field,
    )
    .to_result()
    {
        Ok(discriminator) => Valid::succeed(discriminator),
        Err(e) => Valid::from_validation_err(BlueprintError::from_validation_string(e)),
    }
}

pub fn update_union_resolver<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a Type, &'a str), FieldDefinition, BlueprintError>
{
    TryFold::<(&ConfigModule, &Field, &Type, &str), FieldDefinition, BlueprintError>::new(
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
