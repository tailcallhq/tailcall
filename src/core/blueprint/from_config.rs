use std::collections::{BTreeMap, BTreeSet};

use async_graphql::dynamic::SchemaBuilder;
use indexmap::IndexMap;
use tailcall_valid::{Valid, ValidationError, Validator};

use self::telemetry::to_opentelemetry;
use super::Server;
use crate::core::blueprint::compress::compress;
use crate::core::blueprint::*;
use crate::core::config::transformer::Required;
use crate::core::config::{Arg, Batch, Config, ConfigModule};
use crate::core::ir::model::{IO, IR};
use crate::core::json::JsonSchema;
use crate::core::try_fold::TryFold;
use crate::core::Type;

pub fn config_blueprint<'a>() -> TryFold<'a, ConfigModule, Blueprint, BlueprintError> {
    let server = TryFoldConfig::<Blueprint>::new(|config_module, blueprint| {
        Valid::from(Server::try_from(config_module.clone())).map(|server| blueprint.server(server))
    });

    let schema = to_schema().transform::<Blueprint>(
        |schema, blueprint| blueprint.schema(schema),
        |blueprint| blueprint.schema,
    );

    let definitions = to_definitions().transform::<Blueprint>(
        |definitions, blueprint| blueprint.definitions(definitions),
        |blueprint| blueprint.definitions,
    );

    let upstream = TryFoldConfig::<Blueprint>::new(|config_module, blueprint| {
        Valid::from(Upstream::try_from(config_module)).map(|upstream| blueprint.upstream(upstream))
    });

    let links = TryFoldConfig::<Blueprint>::new(|config_module, blueprint| {
        Valid::from(Links::try_from(config_module.links.clone())).map_to(blueprint)
    });

    let opentelemetry = to_opentelemetry().transform::<Blueprint>(
        |opentelemetry, blueprint| blueprint.telemetry(opentelemetry),
        |blueprint| blueprint.telemetry,
    );

    server
        .and(schema)
        .and(definitions)
        .and(upstream)
        .and(links)
        .and(opentelemetry)
        // set the federation config only after setting other properties to be able
        // to use blueprint inside the handler and to avoid recursion overflow
        .and(update_federation().trace("federation"))
        .update(apply_batching)
        .update(compress)
}

// Apply batching if any of the fields have a @http directive with groupBy field

pub fn apply_batching(mut blueprint: Blueprint) -> Blueprint {
    for def in blueprint.definitions.iter() {
        if let Definition::Object(object_type_definition) = def {
            for field in object_type_definition.fields.iter() {
                if let Some(IR::IO(IO::Http { group_by: Some(_), .. })) = field.resolver.clone() {
                    blueprint.upstream.batch = blueprint.upstream.batch.or(Some(Batch::default()));
                    return blueprint;
                }
            }
        }
    }
    blueprint
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
                    if field.resolvers.is_empty() {
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
    type Error = ValidationError<crate::core::blueprint::BlueprintError>;

    fn try_from(config_module: &ConfigModule) -> Result<Self, Self::Error> {
        config_blueprint()
            .try_fold(
                // Apply required transformers to the configuration
                &config_module
                    .to_owned()
                    .transform(Required)
                    .to_result()
                    .map_err(BlueprintError::from_validation_string)?,
                Blueprint::default(),
            )
            .and_then(|blueprint| {
                let schema_builder = SchemaBuilder::from(&blueprint);
                match schema_builder.finish() {
                    Ok(_) => Valid::succeed(blueprint),
                    Err(e) => Valid::fail(e.into()),
                }
            })
            .to_result()
    }
}
