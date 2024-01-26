use crate::blueprint::*;
use crate::config;
use crate::config::Field;
use crate::lambda::Lambda;
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn update_js<'a>(
) -> TryFold<'a, (&'a ConfigSet, &'a Field, &'a config::Type, &'a str), FieldDefinition, String> {
    TryFold::<(&ConfigSet, &Field, &config::Type, &str), FieldDefinition, String>::new(
        |(_, field, _, _), b_field| {
            let mut updated_b_field = b_field;
            if let Some(op) = &field.script {
                updated_b_field = updated_b_field
                    .resolver_or_default(Lambda::context().to_js(op.script.clone()), |r| {
                        r.to_js(op.script.clone())
                    });
            }
            Valid::succeed(updated_b_field)
        },
    )
}
