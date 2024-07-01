use std::collections::{HashMap, HashSet};

use oas3::spec::ObjectOrReference;
use oas3::{OpenApiV3Spec, Schema};

use crate::core::config::{Config, Enum, Field, Type, Union, Variant};
use crate::core::generator::openapi::helpers::{
    get_schema_type, name_from_ref_path, schema_to_primitive_type, unknown_type,
};
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

fn get_all_of_properties(
    spec: &OpenApiV3Spec,
    properties: &mut Vec<(String, ObjectOrReference<Schema>)>,
    required: &mut HashSet<String>,
    schema: Schema,
) {
    required.extend(schema.required);
    if !schema.all_of.is_empty() {
        for obj in schema.all_of {
            let schema = obj.resolve(spec).unwrap();
            get_all_of_properties(spec, properties, required, schema);
        }
    }
    properties.extend(schema.properties);
}

impl<'a> TypeGenerator<'a> {
    #[allow(clippy::too_many_arguments)]
    fn define_type(
        &self,
        config: &mut Config,
        name: String,
        schema: Schema,
        types: &mut HashMap<String, Schema>,
    ) -> Valid<(), String> {
        if !schema.properties.is_empty() {
            Valid::from_iter(schema.properties, |(name, property)| {
                let property_schema = match property.resolve(self.spec) {
                    Ok(schema) => schema,
                    Err(err) => return Valid::fail(err.to_string()),
                };

                let type_name = get_schema_type(
                    self.spec,
                    property_schema.clone(),
                    name_from_ref_path(&property),
                    types,
                );
                let (list, type_of) = match type_name {
                    Ok(type_name) => type_name.into_tuple(),
                    Err(err) => return Valid::fail(err.to_string()),
                };

                let doc = property_schema.description.clone();
                Valid::succeed((
                    name.clone(),
                    Field {
                        type_of,
                        required: schema.required.contains(&name),
                        list,
                        doc,
                        ..Default::default()
                    },
                ))
            })
            .map(|fields| {
                config.types.insert(
                    name,
                    Type {
                        fields: fields.into_iter().collect(),
                        doc: schema.description.clone(),
                        ..Default::default()
                    },
                );
            })
        } else if !schema.all_of.is_empty() {
            let mut properties: Vec<_> = vec![];
            let mut required = HashSet::new();
            let doc = schema.description.clone();
            get_all_of_properties(self.spec, &mut properties, &mut required, schema);

            Valid::from_iter(properties, |(name, property)| {
                let type_name = get_schema_type(
                    self.spec,
                    property.resolve(self.spec).unwrap(),
                    name_from_ref_path(&property),
                    types,
                );

                let (list, type_of) = match type_name {
                    Ok(val) => val.into_tuple(),
                    Err(err) => return Valid::fail(err.to_string()),
                };

                Valid::succeed((
                    name.clone(),
                    Field {
                        type_of,
                        list,
                        required: required.contains(&name),
                        ..Default::default()
                    },
                ))
            })
            .map(|fields| {
                let fields = fields.into_iter().collect();
                config
                    .types
                    .insert(name, Type { fields, doc, ..Default::default() });
            })
        } else if !schema.any_of.is_empty() || !schema.one_of.is_empty() {
            Valid::from_iter(schema.any_of.iter().chain(schema.one_of.iter()), |schema| {
                let type_name = name_from_ref_path(schema).or_else(|| {
                    schema_to_primitive_type(
                        schema.resolve(self.spec).unwrap().schema_type.as_ref()?,
                    )
                });

                if let Some(type_name) = type_name {
                    return Valid::succeed(type_name);
                }

                match schema.resolve(self.spec) {
                    Ok(schema) => Valid::succeed(unknown_type(types, schema)),
                    Err(err) => Valid::fail(err.to_string()),
                }
            })
            .map(|types| {
                config.unions.insert(
                    name,
                    Union { types: types.into_iter().collect(), doc: schema.description },
                );
            })
        } else if !schema.enum_values.is_empty() {
            Valid::from_iter(schema.enum_values, |val| match val {
                serde_yaml::Value::String(string) => Valid::succeed(string),
                _ => Valid::fail("Enum values must be strings".to_string()),
            })
            .map(|variants| {
                let variants = variants
                    .into_iter()
                    .map(|name| Variant { name, alias: None })
                    .collect();
                config
                    .enums
                    .insert(name, Enum { variants, doc: schema.description });
            })
        } else {
            return Valid::fail("Unable to define type".to_string());
        }
    }
}

impl<'a> Transform for TypeGenerator<'a> {
    type Value = (HashMap<String, Schema>, Config);
    type Error = String;

    fn transform(&self, (mut types, mut config): Self::Value) -> Valid<Self::Value, Self::Error> {
        if let Some(components) = self.spec.components.as_ref() {
            Valid::from_iter(components.schemas.clone(), |(name, obj_or_ref)| {
                let schema = match obj_or_ref.resolve(self.spec) {
                    Ok(schema) => schema,
                    Err(err) => return Valid::fail(err.to_string()),
                };
                if let Err(e) = self
                    .define_type(&mut config, name.clone(), schema, &mut types)
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
