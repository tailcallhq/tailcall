use std::collections::{BTreeMap, HashMap};

use super::{Server, TypeLike};
use crate::blueprint::compress::compress;
use crate::blueprint::*;
use crate::config::{Arg, Batch, Config, ConfigSet, Field, Link};
use crate::directive::DirectiveCodec;
use crate::json::JsonSchema;
use crate::lambda::{Expression, IO};
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError, Validator};

pub fn config_blueprint<'a>() -> TryFold<'a, ConfigSet, Blueprint, String> {
    let server = TryFoldConfig::<Blueprint>::new(|config_set, blueprint| {
        Valid::from(Server::try_from(config_set.server.clone()))
            .map(|server| blueprint.server(server))
    });

    let schema = to_schema().transform::<Blueprint>(
        |schema, blueprint| blueprint.schema(schema),
        |blueprint| blueprint.schema,
    );

    let definitions = to_definitions().transform::<Blueprint>(
        |definitions, blueprint| blueprint.definitions(definitions),
        |blueprint| blueprint.definitions,
    );

    let upstream = TryFoldConfig::<Blueprint>::new(|config_set, blueprint| {
        Valid::from(Upstream::try_from(config_set.upstream.clone()))
            .map(|upstream| blueprint.upstream(upstream))
    });

    let validate_links = TryFoldConfig::<Blueprint>::new(|config_set, blueprint| {
        let links: Vec<Link> = config_set.links.clone();

        if links.iter().any(|link| link.src.is_empty()) {
            return Valid::fail("Link src cannot be empty".to_string())
                .trace(Link::trace_name().as_str())
                .trace("schema");
        }

        Valid::from_iter(links.iter(), |link| {
            let pos = links.iter().position(|l| l == link).unwrap().to_string();

            if link.src.is_empty() {
                return Valid::fail("Link src cannot be empty".to_string()).trace(&pos);
            }

            if let Some(id) = &link.id {
                if links.iter().filter(|l| l.id.as_ref() == Some(id)).count() > 1 {
                    Valid::fail(format!("Duplicated id: {}", id)).trace(&pos)
                } else {
                    Valid::succeed(())
                }
            } else {
                Valid::succeed(())
            }
        })
        .trace(Link::trace_name().as_str())
        .trace("schema")
        .map_to(blueprint)
    });

    server
        .and(schema)
        .and(definitions)
        .and(upstream)
        .and(validate_links)
        .update(apply_batching)
        .update(compress)
}

// Apply batching if any of the fields have a @http directive with groupBy field

pub fn apply_batching(mut blueprint: Blueprint) -> Blueprint {
    for def in blueprint.definitions.iter() {
        if let Definition::ObjectTypeDefinition(object_type_definition) = def {
            for field in object_type_definition.fields.iter() {
                if let Some(Expression::IO(IO::Http { group_by: Some(_), .. })) =
                    field.resolver.clone()
                {
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
    let schema = match type_ {
        Some(type_) => {
            let mut schema_fields = HashMap::new();
            for (name, field) in type_.fields.iter() {
                if field.script.is_none() && field.http.is_none() {
                    schema_fields.insert(name.clone(), to_json_schema_for_field(field, config));
                }
            }
            JsonSchema::Obj(schema_fields)
        }
        None => match type_of {
            "String" => JsonSchema::Str {},
            "Int" => JsonSchema::Num {},
            "Boolean" => JsonSchema::Bool {},
            "JSON" => JsonSchema::Obj(HashMap::new()),
            _ => JsonSchema::Str {},
        },
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

impl TryFrom<&ConfigSet> for Blueprint {
    type Error = ValidationError<String>;

    fn try_from(config_set: &ConfigSet) -> Result<Self, Self::Error> {
        config_blueprint()
            .try_fold(config_set, Blueprint::default())
            .to_result()
    }
}
