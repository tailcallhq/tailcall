use tailcall_valid::{Valid, Validator};

use crate::core::blueprint::{Auth, BlueprintError, FieldDefinition, Provider};
use crate::core::config::{self, ConfigModule, Field};
use crate::core::ir::model::IR;
use crate::core::try_fold::TryFold;

pub fn update_protected<'a>(
    type_name: &'a str,
) -> TryFold<
    'a,
    (&'a ConfigModule, &'a Field, &'a config::Type, &'a str),
    FieldDefinition,
    BlueprintError,
> {
    TryFold::<(&ConfigModule, &Field, &config::Type, &'a str), FieldDefinition, BlueprintError>::new(
        |(config, field, type_, _), mut b_field| {
            if field.protected.is_some() // check the field itself has marked as protected
                || type_.protected.is_some() // check the type that contains current field
                || config // check that output type of the field is protected
                    .find_type(field.type_of.name())
                    .and_then(|type_| type_.protected.as_ref())
                    .is_some()
            {
                if config.input_types().contains(type_name) {
                    return Valid::fail(BlueprintError::InputTypesCannotBeProtected);
                }

                if !config.extensions().has_auth() {
                    return Valid::fail(BlueprintError::ProtectedOperatorNoAuthProviders);
                }

                // Used to collect the providers that are used in the field
                let providers: std::collections::HashMap<_, _> = Provider::from_config(config)
                    .into_iter()
                    .filter_map(|provider| provider.id.clone().map(|id| (id, provider.content)))
                    .collect();

                // FIXME: add trace information in the error

                let mut protection = Vec::new();

                protection.extend(
                    type_
                        .protected
                        .clone()
                        .and_then(|protect| protect.id)
                        .unwrap_or_default(),
                );

                protection.extend(
                    field
                        .protected
                        .clone()
                        .and_then(|protect| protect.id)
                        .unwrap_or_default(),
                );

                Valid::from_iter(protection.iter(), |id| {
                    if let Some(provider) = providers.get(id) {
                        Valid::succeed(Auth::Provider(provider.clone()))
                    } else {
                        Valid::fail(BlueprintError::AuthProviderNotFound(id.clone()))
                    }
                })
                .map(|provider| {
                    let mut auth = provider.into_iter().reduce(|left, right| left.and(right));

                    // If no protection is defined, use all providers
                    if auth.is_none() {
                        auth = Auth::from_config(config);
                    }

                    if let Some(auth) = auth {
                        b_field.resolver = match &b_field.resolver {
                            None => Some(IR::Protect(
                                auth,
                                Box::new(IR::ContextPath(vec![b_field.name.clone()])),
                            )),
                            Some(resolver) => Some(IR::Protect(auth, Box::new(resolver.clone()))),
                        }
                    }

                    b_field
                })
            } else {
                Valid::succeed(b_field)
            }
        },
    )
}
