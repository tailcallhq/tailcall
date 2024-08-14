use std::collections::{BTreeMap, BTreeSet, HashMap};

use async_graphql::dynamic::SchemaBuilder;

use self::telemetry::to_opentelemetry;
use super::{Server, TypeLike};
use crate::core::blueprint::compress::compress;
use crate::core::blueprint::*;
use crate::core::config::transformer::Required;
use crate::core::config::{Arg, Batch, Config, ConfigModule, Field};
use crate::core::ir::model::{IO, IR};
use crate::core::json::JsonSchema;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, ValidationError, Validator};

pub fn config_blueprint<'a>() -> TryFold<'a, ConfigModule, Blueprint, String> {
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

pub fn to_json_schema_for_field(field: &Field, config: &Config) -> JsonSchema {
    to_json_schema(field, config)
}
pub fn to_json_schema_for_args(args: &BTreeMap<String, Arg>, config: &Config) -> JsonSchema {
    let mut schema_fields = HashMap::new();
    for (name, arg) in args.iter() {
        schema_fields.insert(name.clone(), to_json_schema(arg, config));
    }
    JsonSchema::Obj(schema_fields)
}
fn to_json_schema<T>(field: &T, config: &Config) -> JsonSchema
where
    T: TypeLike,
{
    let type_of = field.name();
    let list = field.list();
    let required = field.non_null();
    let type_ = config.find_type(type_of);
    let type_enum_ = config.find_enum(type_of);
    let schema = if let Some(type_) = type_ {
        let mut schema_fields = HashMap::new();
        for (name, field) in type_.fields.iter() {
            if field.script.is_none() && field.http.is_none() {
                schema_fields.insert(name.clone(), to_json_schema_for_field(field, config));
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
        JsonSchema::from_scalar_type(type_of)
    };

    if !required {
        if list {
            JsonSchema::Opt(Box::new(JsonSchema::Arr(Box::new(schema))))
        } else {
            JsonSchema::Opt(Box::new(schema))
        }
    } else if list {
        JsonSchema::Arr(Box::new(schema))
    } else {
        schema
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
