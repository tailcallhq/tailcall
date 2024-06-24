use crate::core::blueprint::*;
use crate::core::config;
use crate::core::config::position::Pos;
use crate::core::config::Field;
use crate::core::directive::DirectiveCodec;
use crate::core::ir::model::IR;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, Validator};

pub fn update_modify<'a>() -> TryFold<
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
        |(config, field, type_of, _), mut b_field| {
            if let Some(modify) = field.modify.as_ref() {
                if let Some(new_name) = &modify.name {
                    for name in type_of.implements.iter() {
                        let interface = config.find_type(name);
                        if let Some(interface) = interface {
                            if interface.fields.iter().any(|(name, _)| name == new_name) {
                                return Valid::fail(
                                    "Field is already implemented from interface".to_string(),
                                ).trace(modify.to_pos_trace_err(config::Modify::trace_name()).as_deref());
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
