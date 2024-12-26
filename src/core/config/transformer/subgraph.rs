use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::convert::identity;
use std::fmt::{Display, Write};
use std::ops::Deref;

use tailcall_macros::MergeRight;
use tailcall_valid::{Valid, Validator};

use crate::core::config::directive::to_directive;
use crate::core::config::{
    self, ApolloFederation, Arg, Call, Config, Field, GraphQL, Grpc, Http, Key, KeyValue, Resolver,
    Union,
};
use crate::core::directive::DirectiveCodec;
use crate::core::merge_right::MergeRight;
use crate::core::mustache::Segment;
use crate::core::{Mustache, Transform, Type};

const ENTITIES_FIELD_NAME: &str = "_entities";
const SERVICE_FIELD_NAME: &str = "_service";
const SERVICE_TYPE_NAME: &str = "_Service";
const UNION_ENTITIES_NAME: &str = "_Entity";
const ENTITIES_ARG_NAME: &str = "representations";
const ENTITIES_TYPE_NAME: &str = "_Any";

/// Adds compatibility layer for Apollo Federation v2
/// so tailcall may act as a Federation Subgraph.
/// Followed by [spec](https://www.apollographql.com/docs/federation/subgraph-spec/)
pub struct Subgraph;

impl Transform for Subgraph {
    type Value = Config;

    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        if !config.server.get_enable_federation() {
            // if federation is disabled don't process the config
            return Valid::succeed(config);
        }
        let config_types = config.types.clone();
        let mut resolver_by_type = BTreeMap::new();

        let valid = Valid::from_iter(config.types.iter_mut(), |(type_name, ty)| {
            if ty.resolvers.len() > 1 {
                // TODO: should support multiple different resolvers actually, see https://www.apollographql.com/docs/graphos/schema-design/federated-schemas/entities/define-keys#multiple-keys
                return Valid::fail(
                    "Only single resolver for entity is currently supported".to_string(),
                );
            }

            if let Some(resolver) = ty.resolvers.first() {
                resolver_by_type.insert(type_name.clone(), resolver.clone());

                KeysExtractor::validate(&config_types, resolver, type_name).and_then(|_| {
                    KeysExtractor::extract_keys(resolver).and_then(|fields| match fields {
                        Some(fields) => {
                            let key = Key { fields };

                            to_directive(key.to_directive()).map(|directive| {
                                // Prevent transformer to push the same directive multiple times
                                if !ty.directives.iter().any(|d| {
                                    d.name == directive.name && d.arguments == directive.arguments
                                }) {
                                    ty.directives.push(directive);
                                }
                            })
                        }
                        None => Valid::succeed(()),
                    })
                })
            } else {
                Valid::succeed(())
            }
            .trace(type_name)
        });

        if valid.is_fail() {
            return valid.map_to(config);
        }

        let service_field = Field { type_of: "String".to_string().into(), ..Default::default() };

        let service_type = config::Type {
            fields: [("sdl".to_string(), service_field)].into_iter().collect(),
            ..Default::default()
        };

        // type that represents the response for service
        config
            .types
            .insert(SERVICE_TYPE_NAME.to_owned(), service_type);

        let query_type_name = match config.schema.query.as_ref() {
            Some(name) => name,
            None => {
                config.schema.query = Some("Query".to_string());
                "Query"
            }
        };

        let query_type = config.types.entry(query_type_name.to_owned()).or_default();

        query_type.fields.insert(
            SERVICE_FIELD_NAME.to_string(),
            Field {
                type_of: Type::from(SERVICE_TYPE_NAME.to_owned()).into_required(),
                doc: Some("Apollo federation Query._service resolver".to_string()),
                resolvers: Resolver::ApolloFederation(ApolloFederation::Service).into(),
                ..Default::default()
            },
        );

        if !resolver_by_type.is_empty() {
            let entity_union = Union {
                types: resolver_by_type.keys().cloned().collect(),
                ..Default::default()
            };

            let entity_resolver = config::EntityResolver { resolver_by_type };

            // union that wraps any possible types for entities
            config
                .unions
                .insert(UNION_ENTITIES_NAME.to_owned(), entity_union);
            // any scalar for argument `representations`
            config
                .types
                .insert(ENTITIES_TYPE_NAME.to_owned(), config::Type::default());

            let query_type = config.types.entry(query_type_name.to_owned()).or_default();

            let arg = Arg {
                type_of: Type::from(ENTITIES_TYPE_NAME.to_string())
                    .into_required()
                    .into_list()
                    .into_required(),
                ..Default::default()
            };

            query_type.fields.insert(
                ENTITIES_FIELD_NAME.to_string(),
                Field {
                    type_of: Type::from(UNION_ENTITIES_NAME.to_owned())
                        .into_list()
                        .into_required(),
                    args: [(ENTITIES_ARG_NAME.to_owned(), arg)].into_iter().collect(),
                    doc: Some("Apollo federation Query._entities resolver".to_string()),
                    resolvers: Resolver::ApolloFederation(ApolloFederation::EntityResolver(
                        entity_resolver,
                    ))
                    .into(),
                    ..Default::default()
                },
            );
        }

        Valid::succeed(config)
    }
}

#[derive(Default, Clone, Debug, MergeRight)]
struct Keys(BTreeMap<String, Keys>);

impl Keys {
    fn new() -> Self {
        Self::default()
    }

    fn set_path(&mut self, path: impl Iterator<Item = String>) {
        let mut map = &mut self.0;

        for part in path {
            map = &mut map.entry(part).or_default().0;
        }
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Display for Keys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, (key, value)) in self.0.iter().enumerate() {
            f.write_str(key)?;

            if !value.0.is_empty() {
                write!(f, " {{ {} }}", value)?;
            }

            if i < self.0.len() - 1 {
                f.write_char(' ')?;
            }
        }

        Ok(())
    }
}

fn combine_keys(v: Vec<Keys>) -> Keys {
    v.into_iter()
        .fold(Keys::new(), |acc, keys| acc.merge_right(keys))
}

struct KeysExtractor;

impl KeysExtractor {
    fn validate_expressions<'a>(
        type_name: &str,
        type_map: &BTreeMap<String, config::Type>,
        expr_iter: impl Iterator<Item = &'a Segment>,
    ) -> Valid<(), String> {
        Valid::from_iter(expr_iter, |segment| {
            if let Segment::Expression(expr) = segment {
                if expr.len() > 1 && expr[0].as_str() == "value" {
                    Self::validate_iter(type_map, type_name, expr.iter().skip(1))
                } else {
                    Valid::succeed(())
                }
            } else {
                Valid::succeed(())
            }
        })
        .unit()
    }

    fn validate_iter<'a>(
        type_map: &BTreeMap<String, config::Type>,
        current_type: &str,
        fields_iter: impl Iterator<Item = &'a String>,
    ) -> Valid<(), String> {
        let mut current_type = current_type;
        Valid::from_iter(fields_iter.enumerate(), |(index, key)| {
            if let Some(type_def) = type_map.get(current_type) {
                if !type_def.fields.contains_key(key) {
                    return Valid::fail(format!(
                        "Invalid key at index {}: '{}' is not a field of '{}'",
                        index, key, current_type
                    ));
                }
                current_type = type_def.fields[key].type_of.name();
            } else {
                return Valid::fail(format!("Type '{}' not found in config", current_type));
            }
            Valid::succeed(())
        })
        .unit()
    }

    fn validate(
        type_map: &BTreeMap<String, config::Type>,
        resolver: &Resolver,
        type_name: &str,
    ) -> Valid<(), String> {
        if let Resolver::Http(http) = resolver {
            Valid::from_iter(http.query.iter(), |q| {
                Self::validate_expressions(
                    type_name,
                    type_map,
                    Mustache::parse(&q.value).segments().iter(),
                )
            })
            .and(Self::validate_expressions(
                type_name,
                type_map,
                Mustache::parse(&http.url).segments().iter(),
            ))
            .unit()
        } else {
            Valid::succeed(())
        }
    }

    fn extract_keys(resolver: &Resolver) -> Valid<Option<String>, String> {
        // TODO: add validation for available fields from the type
        match resolver {
            Resolver::Http(http) => {
                Valid::from_iter(
                    [
                        Self::parse_str(http.url.as_str()).trace("url"),
                        Self::parse_json_option(http.body.as_ref()).trace("body"),
                        Self::parse_key_value_iter(http.headers.iter()).trace("headers"),
                        Self::parse_key_value_iter(http.query.iter().map(|q| KeyValue {
                            key: q.key.to_string(),
                            value: q.value.to_string(),
                        }))
                        .trace("query"),
                    ],
                    identity,
                )
                .trace(Http::directive_name().as_str())
            }
            Resolver::Grpc(grpc) => Valid::from_iter(
                [
                    Self::parse_str(grpc.url.as_str()),
                    Self::parse_str(&grpc.method),
                    Self::parse_value_option(&grpc.body),
                    Self::parse_key_value_iter(grpc.headers.iter()),
                ],
                identity,
            )
            .trace(Grpc::directive_name().as_str()),
            Resolver::Graphql(graphql) => Valid::from_iter(
                [
                    Self::parse_key_value_iter(graphql.headers.iter()),
                    Self::parse_key_value_iter_option(graphql.args.as_ref().map(|v| v.iter())),
                ],
                identity,
            )
            .trace(GraphQL::directive_name().as_str()),
            Resolver::Call(call) => Valid::from_option(
                call.steps.first(),
                "Call should define at least one step".to_string(),
            )
            .and_then(|step| {
                Valid::from_iter(step.args.iter(), |(key, value)| {
                    Valid::from_iter([Self::parse_str(key), Self::parse_value(value)], identity)
                })
            })
            .map(|v| v.into_iter().flatten().collect())
            .trace(Call::directive_name().as_str()),
            Resolver::Expr(expr) => Valid::from_iter([Self::parse_value(&expr.body)], identity)
                .trace(Call::directive_name().as_str()),
            _ => return Valid::succeed(None),
        }
        .map(|keys| {
            let keys = combine_keys(keys);

            if keys.is_empty() {
                None
            } else {
                Some(keys.to_string())
            }
        })
    }

    fn parse_str(s: &str) -> Valid<Keys, String> {
        let mustache = Mustache::parse(s);
        let mut keys = Keys::new();

        Valid::from_iter(mustache.segments().iter(), |segment| {
            if let Segment::Expression(expr) = segment {
                match expr.first().map(Deref::deref) {
                    Some("value") => {
                        keys.set_path(expr[1..].iter().map(String::to_string));
                    }
                    Some("args") => {
                        return Valid::fail(
                            "Type resolver can't use `.args`, use `.value` instead".to_string(),
                        );
                    }
                    _ => {}
                }
            }

            Valid::succeed(())
        })
        .map_to(keys)
    }

    fn parse_json_option(s: Option<&serde_json::Value>) -> Valid<Keys, String> {
        if let Some(s) = s {
            Self::parse_str(&s.to_string())
        } else {
            Valid::succeed(Keys::new())
        }
    }

    fn parse_key_value_iter<T: Borrow<KeyValue>>(
        it: impl Iterator<Item = T>,
    ) -> Valid<Keys, String> {
        let mut keys = Keys::new();

        Valid::from_iter(it, |key_value| {
            let key_value = key_value.borrow();

            Self::parse_str(&key_value.key)
                .zip(Self::parse_str(&key_value.value))
                .map(|(key, value)| keys = keys.clone().merge_right(key).merge_right(value))
        })
        .map_to(keys)
    }

    fn parse_key_value_iter_option<T: Borrow<KeyValue>>(
        it: Option<impl Iterator<Item = T>>,
    ) -> Valid<Keys, String> {
        if let Some(it) = it {
            Self::parse_key_value_iter(it)
        } else {
            Valid::succeed(Keys::new())
        }
    }

    fn parse_value(value: &serde_json::Value) -> Valid<Keys, String> {
        match value {
            serde_json::Value::String(s) => return Self::parse_str(s),
            serde_json::Value::Array(v) => Valid::from_iter(v.iter(), Self::parse_value),
            serde_json::Value::Object(map) => Valid::from_iter(map.iter(), |(k, v)| {
                Self::parse_str(k)
                    .zip(Self::parse_value(v))
                    .map(|(k, v)| k.merge_right(v))
            }),
            _ => return Valid::succeed(Keys::new()),
        }
        .map(|keys_vec| {
            keys_vec
                .into_iter()
                .fold(Keys::new(), |acc, keys| acc.merge_right(keys))
        })
    }

    fn parse_value_option(value: &Option<serde_json::Value>) -> Valid<Keys, String> {
        if let Some(value) = value {
            Self::parse_value(value)
        } else {
            Valid::succeed(Keys::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod keys {
        use insta::assert_debug_snapshot;

        use super::*;

        #[test]
        fn test_keys_set() {
            let mut keys = Keys::new();

            keys.set_path(["a", "b", "c"].into_iter().map(str::to_string));
            keys.set_path(["a", "d"].into_iter().map(str::to_string));
            keys.set_path(["e"].into_iter().map(str::to_string));
            keys.set_path(["f", "g"].into_iter().map(str::to_string));

            assert_debug_snapshot!(keys);
        }

        #[test]
        fn test_keys_merge() {
            let mut keys_a = Keys::new();

            keys_a.set_path(["a", "b"].into_iter().map(str::to_string));
            keys_a.set_path(["c"].into_iter().map(str::to_string));

            let mut keys_b = Keys::new();

            keys_b.set_path(["a", "1"].into_iter().map(str::to_string));
            keys_b.set_path(["c", "2"].into_iter().map(str::to_string));
            keys_b.set_path(["d", "3"].into_iter().map(str::to_string));

            assert_debug_snapshot!(keys_a.merge_right(keys_b));
        }
    }

    #[cfg(test)]
    mod extractor {
        use insta::assert_debug_snapshot;
        use serde_json::json;

        use super::config::Http;
        use super::{KeyValue, KeysExtractor, Resolver};
        use crate::core::config::{Call, Expr, GraphQL, Grpc, Step, URLQuery};
        use crate::core::http::Method;

        #[test]
        fn test_non_value_template() {
            let http = Http {
                url: "http://tailcall.run/users/{{.args.id}}".to_string(),
                query: vec![URLQuery {
                    key: "{{.env.query.key}}".to_string(),
                    value: "{{.args.query.value}}".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            };
            let resolver = Resolver::Http(http);
            let keys = KeysExtractor::extract_keys(&resolver);

            assert_debug_snapshot!(keys);
        }

        #[test]
        fn test_extract_http() {
            let http = Http {
                url: "http://tailcall.run/users/{{.value.id}}".to_string(),
                body: Some(serde_json::Value::String(
                    r#"{ "obj": "{{.value.obj}}"} "#.to_string(),
                )),
                headers: vec![KeyValue {
                    key: "{{.value.header.key}}".to_string(),
                    value: "{{.value.header.value}}".to_string(),
                }],
                method: Method::POST,
                query: vec![URLQuery {
                    key: "{{.value.query_key}}".to_string(),
                    value: "{{.value.query_value}}".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            };
            let resolver = Resolver::Http(http);
            let keys = KeysExtractor::extract_keys(&resolver);

            assert_debug_snapshot!(keys);
        }

        #[test]
        fn test_extract_grpc() {
            let grpc = Grpc {
                url: "http://localhost:5051/{{.env.target}}".to_string(),
                body: Some(json!({ "a": "{{.value.body.a}}", "b": "{{.value.body.b}}"})),
                headers: vec![KeyValue {
                    key: "test".to_string(),
                    value: "{{.value.header_test}}".to_string(),
                }],
                method: "test_{{.value.method}}".to_string(),
                ..Default::default()
            };

            let resolver = Resolver::Grpc(grpc);
            let keys = KeysExtractor::extract_keys(&resolver);

            assert_debug_snapshot!(keys);
        }

        #[test]
        fn test_extract_graphql() {
            let graphql = GraphQL {
                url: "http://localhost:5051/{{.env.target}}".to_string(),
                headers: vec![KeyValue {
                    key: "test".to_string(),
                    value: "{{.value.header_test}}".to_string(),
                }],
                args: Some(vec![KeyValue {
                    key: "key".to_string(),
                    value: "test-{{.value.input.key}}".to_string(),
                }]),
                ..Default::default()
            };

            let resolver = Resolver::Graphql(graphql);
            let keys = KeysExtractor::extract_keys(&resolver);

            assert_debug_snapshot!(keys);
        }

        #[test]
        fn test_extract_call() {
            let call = Call {
                steps: vec![Step {
                    query: Some("field".to_string()),
                    args: [(
                        "arg".to_string(),
                        json!(json!({ "a": "{{.value.arg.a}}", "b": "{{.value.arg.b}}"})),
                    )]
                    .into_iter()
                    .collect(),
                    ..Default::default()
                }],
                dedupe: None,
            };

            let resolver = Resolver::Call(call);
            let keys = KeysExtractor::extract_keys(&resolver);

            assert_debug_snapshot!(keys);
        }

        #[test]
        fn test_extract_expr() {
            let expr = Expr {
                body: json!({ "a": "{{.value.body.a}}", "b": "{{.value.body.b}}"}),
            };

            let resolver = Resolver::Expr(expr);
            let keys = KeysExtractor::extract_keys(&resolver);

            assert_debug_snapshot!(keys);
        }
    }
}
