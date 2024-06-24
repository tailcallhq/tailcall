use crate::core::blueprint::FieldDefinition;
use crate::core::config::position::Pos;
use crate::core::config::{self, ConfigModule, Field};
use crate::core::directive::DirectiveCodec;
use crate::core::ir::model::IR;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, Validator};

pub fn update_protected<'a>(
    type_name: &'a str,
) -> TryFold<
    'a,
    (
        &'a ConfigModule,
        &'a Pos<Field>,
        &'a Pos<config::Type>,
        &'a str,
    ),
    FieldDefinition,
    String,
> {
    TryFold::<(&ConfigModule, &Pos<Field>, &Pos<config::Type>, &'a str), FieldDefinition, String>::new(
        |(config, field, type_, _), mut b_field| {

            let protected = if let Some(protected) = field.protected.as_ref() {
                Some(protected)
            } else if let Some(protected) = type_.protected.as_ref() {
                Some(protected)
            } else {
                config
                    .find_type(&field.type_of)
                    .and_then(|type_| type_.protected.as_ref())
            };

            if let Some(protected) = protected {
                if config.input_types.contains(type_name) {
                    return Valid::fail("Input types can not be protected".to_owned())
                        .trace(protected.to_pos_trace_err(config::Protected::trace_name()).as_deref());
                }

                if !config.extensions.has_auth() {
                    return Valid::fail(
                        "@protected operator is used but there is no @link definitions for auth providers".to_owned(),
                    )
                        .trace(protected.to_pos_trace_err(config::Protected::trace_name()).as_deref());
                }

                b_field.resolver = Some(IR::Protect(Box::new(
                    b_field
                        .resolver
                        .unwrap_or(IR::ContextPath(vec![b_field.name.clone()])),
                )));
            }

            Valid::succeed(b_field)
        },
    )
}
