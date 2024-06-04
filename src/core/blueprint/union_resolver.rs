use crate::core::{config::{ConfigModule, Field}, ir::Context};
use crate::core::ir::IR;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, Validator};
use crate::core::{blueprint::FieldDefinition, ir::Discriminator};
use crate::core::{config, valid::ValidationError};

fn compile_union_resolver(
    config: &ConfigModule,
    union_: &config::Union,
) -> Valid<Discriminator, String> {
    Valid::from_iter(&union_.types, |type_| {
        Valid::from_option(
            config.find_type(&type_),
            "Can't find a type that is member of union type".to_string(),
        )
    })
    .and_then(|types| {
        // TODO: do we need to check also addedFields here?
        let fields_by_type = union_
            .types
            .iter()
            .zip(types)
            .map(|(type_name, type_)| (type_name.clone(), type_.fields.keys().cloned().collect()))
            .collect();

        Valid::from(
            Discriminator::new(fields_by_type).map_err(|e| ValidationError::new(e.to_string())),
        )
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

            compile_union_resolver(config, union_).map(|discriminator| {
                b_field.resolver = Some(
                    b_field
                        .resolver
                        .unwrap_or(IR::Context(Context::Path(vec![b_field.name.clone()]))),
                );
                b_field.map_expr(move |expr| IR::Discriminate(discriminator, expr.into()));
                b_field
            })
        },
    )
}
