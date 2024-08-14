use std::collections::BTreeMap;

use crate::core::config::{config, Arg, Config, Field, Resolver, Type, Union};
use crate::core::valid::Valid;
use crate::core::Transform;

const ROOT_FIELD_NAME: &str = "_entities";
const UNION_ENTITIES_NAME: &str = "_Entity";
const ARG_NAME: &str = "representations";
const ARG_TYPE_NAME: &str = "_Any";

pub struct EntityResolver;

impl Transform for EntityResolver {
    type Value = Config;

    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        let mut resolver_by_type = BTreeMap::new();

        for (type_name, ty) in &config.types {
            if let Some(resolver) = &ty.resolver {
                resolver_by_type.insert(type_name.clone(), resolver.clone());
            }
        }

        if resolver_by_type.is_empty() {
            return Valid::succeed(config);
        }

        let entity_union = Union {
            types: resolver_by_type.keys().cloned().collect(),
            ..Default::default()
        };

        // union that wraps any possible types for entities
        config
            .unions
            .insert(UNION_ENTITIES_NAME.to_owned(), entity_union);
        // any scalar for argument `representations`
        config
            .types
            .insert(ARG_TYPE_NAME.to_owned(), Type::default());

        let entity_resolver = config::EntityResolver { resolver_by_type };

        let query_type = match config.schema.query.as_ref() {
            Some(name) => name,
            None => {
                config.schema.query = Some("Query".to_string());
                "Query"
            }
        };

        let query_type = config.types.entry(query_type.to_owned()).or_default();

        let arg = Arg {
            type_of: ARG_TYPE_NAME.to_string(),
            list: true,
            required: true,
            ..Default::default()
        };

        query_type.fields.insert(
            ROOT_FIELD_NAME.to_string(),
            Field {
                type_of: UNION_ENTITIES_NAME.to_owned(),
                list: true,
                required: true,
                args: [(ARG_NAME.to_owned(), arg)].into_iter().collect(),
                doc: Some("Apollo federation Query._entities resolver".to_string()),
                resolver: Some(Resolver::EntityResolver(entity_resolver)),
                ..Default::default()
            },
        );

        Valid::succeed(config)
    }
}
