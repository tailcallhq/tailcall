use std::collections::{BTreeMap, BTreeSet};

use async_graphql::dynamic::SchemaBuilder;
use indexmap::IndexMap;
use tailcall_valid::{Valid, ValidationError, Validator};

use crate::core::blueprint::compress::compress;
use crate::core::blueprint::*;
use crate::core::config::transformer::Required;
use crate::core::config::{Arg, Config, ConfigModule};
use crate::core::json::JsonSchema;
use crate::core::try_fold::TryFold;
use crate::core::Type;

pub fn config_blueprint<'a>() -> TryFold<'a, ConfigModule, Blueprint, String> {
    let schema = to_schema().transform::<Blueprint>(
        |schema, blueprint| blueprint.schema(schema),
        |blueprint| blueprint.schema,
    );

    let definitions = to_definitions().transform::<Blueprint>(
        |definitions, blueprint| blueprint.definitions(definitions),
        |blueprint| blueprint.definitions,
    );

    let links = TryFoldConfig::<Blueprint>::new(|config_module, blueprint| {
        Valid::from(Links::try_from(config_module.links.clone())).map_to(blueprint)
    });

    schema
        .and(definitions)
        .and(links)
        // set the federation config only after setting other properties to be able
        // to use blueprint inside the handler and to avoid recursion overflow
        .and(update_federation().trace("federation"))
        .update(compress)
}

pub fn to_json_schema_for_args(args: &IndexMap<String, Arg>, config: &Config) -> JsonSchema {
    let mut schema_fields = BTreeMap::new();
    for (name, arg) in args.iter() {
        schema_fields.insert(name.clone(), to_json_schema(&arg.type_of, config));
    }
    JsonSchema::Obj(schema_fields)
}
pub fn to_json_schema(type_of: &Type, config: &Config) -> JsonSchema {
    let json_schema = match type_of {
        Type::Named { name, .. } => {
            let type_ = config.find_type(name);
            let type_enum_ = config.find_enum(name);

            if let Some(type_) = type_ {
                let mut schema_fields = BTreeMap::new();
                for (name, field) in type_.fields.iter() {
                    if field.resolver.is_none() {
                        schema_fields.insert(name.clone(), to_json_schema(&field.type_of, config));
                    }
                }
                JsonSchema::Obj(schema_fields)
            } else if let Some(type_enum_) = type_enum_ {
                JsonSchema::Enum(
                    type_enum_
                        .variants
                        .iter()
                        .map(|variant| variant.name.clone())
                        .collect::<BTreeSet<String>>(),
                )
            } else {
                JsonSchema::from_scalar_type(name)
            }
        }
        Type::List { of_type, .. } => JsonSchema::Arr(Box::new(to_json_schema(of_type, config))),
    };

    if type_of.is_nullable() {
        JsonSchema::Opt(Box::new(json_schema))
    } else {
        json_schema
    }
}

impl TryFrom<&ConfigModule> for Blueprint {
    type Error = ValidationError<String>;

    fn try_from(config_module: &ConfigModule) -> Result<Self, Self::Error> {
        config_blueprint()
            .try_fold(
                // Apply required transformers to the configuration
                &config_module.to_owned().transform(Required).to_result()?,
                Blueprint::default(),
            )
            .and_then(|blueprint| {
                let schema_builder = SchemaBuilder::from(&blueprint);
                match schema_builder.finish() {
                    Ok(_) => Valid::succeed(blueprint),
                    Err(e) => Valid::fail(e.to_string()),
                }
            })
            .to_result()
    }
}
