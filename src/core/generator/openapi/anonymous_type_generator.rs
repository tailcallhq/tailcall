use oas3::{OpenApiV3Spec, Schema};

use crate::core::config::Config;
use crate::core::generator::openapi::helpers::define_type;
use crate::core::transform::Transform;
use crate::core::valid::{Valid, Validator};

#[derive(Clone)]
pub struct AnonymousTypes {
    types: Vec<(String, Schema)>,
}

impl AnonymousTypes {
    pub fn new() -> Self {
        Self { types: Vec::new() }
    }

    pub fn add(&mut self, schema: Schema) -> String {
        self.types
            .iter()
            .find_map(|(name, schema1)| schema1.eq(&schema).then_some(name.clone()))
            .unwrap_or_else(|| {
                let name = format!("Type{}", self.types.len());
                self.types.push((name.clone(), schema));
                name
            })
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    pub fn into_iter(self) -> impl Iterator<Item = (String, Schema)> {
        self.types.into_iter()
    }
}

pub struct AnonymousTypeGenerator<'a> {
    spec: &'a OpenApiV3Spec,
}

impl<'a> AnonymousTypeGenerator<'a> {
    pub fn new(spec: &'a OpenApiV3Spec) -> Self {
        Self { spec }
    }
}

impl<'a> Transform for AnonymousTypeGenerator<'a> {
    type Value = (AnonymousTypes, Config);
    type Error = String;

    fn transform(&self, (types, mut config): Self::Value) -> Valid<Self::Value, Self::Error> {
        let mut new_types = AnonymousTypes::new();
        for (name, schema) in types.into_iter() {
            if let Err(err) =
                define_type(self.spec, &mut config, name.clone(), schema, &mut new_types)
                    .to_result()
            {
                tracing::warn!("Failed to define type {}: {}", name, err);
            }
        }
        Valid::succeed((new_types, config))
    }
}
