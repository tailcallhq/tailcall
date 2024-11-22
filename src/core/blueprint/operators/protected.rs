use tailcall_valid::{Valid, Validator};

use crate::core::blueprint::{Auth, FieldDefinition, Provider};
use crate::core::config::{self, ConfigModule, Field};
use crate::core::ir::model::IR;
use crate::core::try_fold::TryFold;

pub fn update_protected<'a>(
    type_name: &'a str,
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
        |(config, field, type_, _), mut b_field| {
            if field.protected.is_some() // check the field itself has marked as protected
                || type_.protected.is_some() // check the type that contains current field
                || config // check that output type of the field is protected
                    .find_type(field.type_of.name())
                    .and_then(|type_| type_.protected.as_ref())
                    .is_some()
            {
                if config.input_types().contains(type_name) {
                    return Valid::fail("Input types can not be protected".to_owned());
                }

                if !config.extensions().has_auth() {
                    return Valid::fail(
                        "@protected operator is used but there is no @link definitions for auth providers".to_owned(),
                    );
                }

                // Used to collect the providers that are used in the field
                Provider::from_config_module(config)
                    .and_then(|config_providers| {
                        Valid::from_iter(field.protected.iter(), |protected_directive| {
                            if let Some(local_field_providers) = &protected_directive.providers {
                                Valid::from_iter(local_field_providers.iter(), |provider_name| {
                                    if let Some(provider) = config_providers.get(provider_name) {
                                        Valid::succeed(Auth::Provider(provider.clone()))
                                    } else {
                                        Valid::fail(format!(
                                            "Auth provider {} not found",
                                            provider_name
                                        ))
                                    }
                                })
                                .map(|auth_providers| {
                                    auth_providers
                                        .into_iter()
                                        .reduce(|left, right| left.and(right))
                                })
                            } else {
                                Valid::succeed(None)
                            }
                        })
                    })
                    .map(|auth_providers| {
                        auth_providers
                            .into_iter()
                            .collect::<Option<Vec<_>>>()
                            .and_then(|auths| {
                                auths.into_iter().reduce(|left, right| left.or(right))
                            })
                    })
                    .map(|auth| {
                        b_field.resolver = Some(IR::Protect(
                            auth,
                            Box::new(
                                b_field
                                    .resolver
                                    .unwrap_or(IR::ContextPath(vec![b_field.name.clone()])),
                            ),
                        ));

                        b_field
                    })
            } else {
                Valid::succeed(b_field)
            }
        },
    )
}
