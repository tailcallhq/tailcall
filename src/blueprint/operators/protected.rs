use crate::blueprint::FieldDefinition;
use crate::config::{self, ConfigModule, Field};
use crate::lambda::{Context, Expression};
use crate::try_fold::TryFold;
use crate::valid::Valid;

pub fn update_protected<'a>(
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
        |(config, field, type_, _), mut b_field| {
            if field.protected.is_some() || type_.protected.is_some() {
                if !config.server.auth.is_some() {
                    return Valid::fail(
                        "@protected operator is used without defining auth @server".to_owned(),
                    );
                }

                b_field.resolver = Some(Expression::Protected(Box::new(
                    b_field
                        .resolver
                        .unwrap_or(Expression::Context(Context::Value)),
                )));
            }

            Valid::succeed(b_field)
        },
    )
}
