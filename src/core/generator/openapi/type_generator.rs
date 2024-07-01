use oas3::OpenApiV3Spec;

use crate::core::config::Config;
use crate::core::generator::openapi::helpers::define_type;
use crate::core::generator::openapi::AnonymousTypes;
use crate::core::transform::Transform;
use crate::core::valid::{Valid, Validator};

pub struct TypeGenerator<'a> {
    spec: &'a OpenApiV3Spec,
}

impl<'a> TypeGenerator<'a> {
    pub(crate) fn new(spec: &'a OpenApiV3Spec) -> Self {
        Self { spec }
    }
}

impl<'a> Transform for TypeGenerator<'a> {
    type Value = (AnonymousTypes, Config);
    type Error = String;

    fn transform(&self, (mut types, mut config): Self::Value) -> Valid<Self::Value, Self::Error> {
        if let Some(components) = self.spec.components.as_ref() {
            Valid::from_iter(components.schemas.clone(), |(name, obj_or_ref)| {
                let schema = match obj_or_ref.resolve(self.spec) {
                    Ok(schema) => schema,
                    Err(err) => return Valid::fail(err.to_string()),
                };
                if let Err(e) =
                    define_type(self.spec, &mut config, name.clone(), schema, &mut types)
                        .to_result()
                {
                    tracing::warn!("Failed to define type {}: {}", name, e);
                };
                Valid::succeed(())
            });
        }
        Valid::succeed((types, config))
    }
}
