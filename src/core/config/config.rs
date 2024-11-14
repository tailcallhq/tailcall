use derive_getters::Getters;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tailcall_macros::MergeRight;
use tailcall_valid::Valid;

use super::{LinkConfig, ServerConfig, Source, TelemetryConfig, UpstreamConfig};
use crate::core::{is_default, merge_right::MergeRight, variance::Invariant};

#[derive(
    Serialize, Deserialize, Clone, Debug, Default, Getters, PartialEq, Eq, JsonSchema, MergeRight,
)]
pub struct Config {
    ///
    /// Dictates how the server behaves and helps tune tailcall for all ingress
    /// requests. Features such as request batching, SSL, HTTP2 etc. can be
    /// configured here.
    pub server: ServerConfig,

    ///
    /// Dictates how tailcall should handle upstream requests/responses.
    /// Tuning upstream can improve performance and reliability for connections.
    pub upstream: UpstreamConfig,

    ///
    /// Linked files, that merge with config, schema or provide metadata.
    pub links: Vec<LinkConfig>,

    /// Enable [opentelemetry](https://opentelemetry.io) support.
    #[serde(default, skip_serializing_if = "is_default")]
    pub telemetry: TelemetryConfig,
}

impl Config {
    pub fn port(&self) -> u16 {
        self.server.port.unwrap_or(8000)
    }

    pub fn to_yaml(&self) -> anyhow::Result<String> {
        Ok(serde_yaml::to_string(self)?)
    }

    pub fn to_json(&self, pretty: bool) -> anyhow::Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }

    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn from_yaml(yaml: &str) -> anyhow::Result<Self> {
        Ok(serde_yaml::from_str(yaml)?)
    }

    pub fn from_source(source: Source, data: &str) -> anyhow::Result<Self> {
        match source {
            Source::Json => Ok(Config::from_json(data)?),
            Source::Yml => Ok(Config::from_yaml(data)?),
        }
    }
}

impl Invariant for Config {
    fn unify(self, other: Self) -> Valid<Self, String> {
        Valid::succeed(self.merge_right(other))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::core::directive::DirectiveCodec;

    #[test]
    fn test_field_has_or_not_batch_resolver() {
        let f1 = Field { ..Default::default() };

        let f2 = Field {
            resolver: Some(Resolver::Http(Http {
                batch_key: vec!["id".to_string()],
                ..Default::default()
            })),
            ..Default::default()
        };

        let f3 = Field {
            resolver: Some(Resolver::Http(Http {
                batch_key: vec![],
                ..Default::default()
            })),
            ..Default::default()
        };

        assert!(!f1.has_batched_resolver());
        assert!(f2.has_batched_resolver());
        assert!(!f3.has_batched_resolver());
    }

    #[test]
    fn test_graphql_directive_name() {
        let name = GraphQL::directive_name();
        assert_eq!(name, "graphQL");
    }

    #[test]
    fn test_from_sdl_empty() {
        let actual = Config::from_sdl("type Foo {a: Int}").to_result().unwrap();
        let expected = Config::default().types(vec![(
            "Foo",
            Type::default().fields(vec![("a", Field::int())]),
        )]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_unused_types_with_cyclic_types() {
        let config = Config::from_sdl(
            "
            type Bar {a: Int}
            type Foo {a: [Foo]}

            type Query {
                foos: [Foo]
            }

            schema {
                query: Query
            }
            ",
        )
        .to_result()
        .unwrap();

        let actual = config.unused_types();
        let mut expected = HashSet::new();
        expected.insert("Bar".to_string());

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_is_root_operation_type_with_query() {
        let mut config = Config::default();
        config.schema.query = Some("Query".to_string());

        assert!(config.is_root_operation_type("Query"));
        assert!(!config.is_root_operation_type("Mutation"));
        assert!(!config.is_root_operation_type("Subscription"));
    }

    #[test]
    fn test_is_root_operation_type_with_mutation() {
        let mut config = Config::default();
        config.schema.mutation = Some("Mutation".to_string());

        assert!(!config.is_root_operation_type("Query"));
        assert!(config.is_root_operation_type("Mutation"));
        assert!(!config.is_root_operation_type("Subscription"));
    }

    #[test]
    fn test_is_root_operation_type_with_subscription() {
        let mut config = Config::default();
        config.schema.subscription = Some("Subscription".to_string());

        assert!(!config.is_root_operation_type("Query"));
        assert!(!config.is_root_operation_type("Mutation"));
        assert!(config.is_root_operation_type("Subscription"));
    }

    #[test]
    fn test_is_root_operation_type_with_no_root_operation() {
        let config = Config::default();

        assert!(!config.is_root_operation_type("Query"));
        assert!(!config.is_root_operation_type("Mutation"));
        assert!(!config.is_root_operation_type("Subscription"));
    }

    #[test]
    fn test_union_types() {
        let sdl = std::fs::read_to_string(tailcall_fixtures::configs::UNION_CONFIG).unwrap();
        let config = Config::from_sdl(&sdl).to_result().unwrap();
        let union_types = config.union_types();
        let expected_union_types: HashSet<String> = ["Bar", "Baz", "Foo"]
            .iter()
            .cloned()
            .map(String::from)
            .collect();
        assert_eq!(union_types, expected_union_types);
    }

    #[test]
    fn test_interfaces_types_map() {
        let sdl = std::fs::read_to_string(tailcall_fixtures::configs::INTERFACE_CONFIG).unwrap();
        let config = Config::from_sdl(&sdl).to_result().unwrap();
        let interfaces_types_map = config.interfaces_types_map();

        let mut expected_union_types = BTreeMap::new();

        {
            let mut set = BTreeSet::new();
            set.insert("E".to_string());
            set.insert("F".to_string());
            expected_union_types.insert("T0".to_string(), set);
        }

        {
            let mut set = BTreeSet::new();
            set.insert("A".to_string());
            set.insert("E".to_string());
            set.insert("B".to_string());
            set.insert("C".to_string());
            set.insert("D".to_string());
            expected_union_types.insert("T1".to_string(), set);
        }

        {
            let mut set = BTreeSet::new();
            set.insert("B".to_string());
            set.insert("E".to_string());
            set.insert("D".to_string());
            expected_union_types.insert("T2".to_string(), set);
        }

        {
            let mut set = BTreeSet::new();
            set.insert("C".to_string());
            set.insert("E".to_string());
            set.insert("D".to_string());
            expected_union_types.insert("T3".to_string(), set);
        }

        {
            let mut set = BTreeSet::new();
            set.insert("D".to_string());
            expected_union_types.insert("T4".to_string(), set);
        }

        assert_eq!(interfaces_types_map, expected_union_types);
    }
}
