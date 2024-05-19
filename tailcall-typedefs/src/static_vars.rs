// Static variables and configurations used in GraphQL schema generation.
use lazy_static::lazy_static;
use crate::entity::Entity;

pub static GRAPHQL_SCHEMA_FILE: &str = "generated/.tailcallrc.graphql";

lazy_static! {
    pub static ref DIRECTIVE_ALLOW_LIST: Vec<(&'static str, Vec<Entity>, bool)> = vec![
        ("server", vec![Entity::Schema], false),
        ("link", vec![Entity::Schema], true),
        ("upstream", vec![Entity::Schema], false),
        ("http", vec![Entity::FieldDefinition], false),
        ("call", vec![Entity::FieldDefinition], false),
        ("grpc", vec![Entity::FieldDefinition], false),
        ("addField", vec![Entity::Object], true),
        ("modify", vec![Entity::FieldDefinition], false),
        ("telemetry", vec![Entity::Schema], false),
        ("omit", vec![Entity::FieldDefinition], false),
        ("groupBy", vec![Entity::FieldDefinition], false),
        ("expr", vec![Entity::FieldDefinition], false),
        (
            "protected",
            vec![Entity::Object, Entity::FieldDefinition],
            false
        ),
        ("graphQL", vec![Entity::FieldDefinition], false),
        (
            "cache",
            vec![Entity::Object, Entity::FieldDefinition],
            false,
        ),
        ("js", vec![Entity::FieldDefinition], false),
        ("tag", vec![Entity::Object], false),
    ];
}

pub static OBJECT_WHITELIST: &[&str] = &[
    "ExprBody",
    "If",
    "Http",
    "Grpc",
    "GraphQL",
    "Proxy",
    "KeyValue",
    "Batch",
    "HttpVersion",
    "Method",
    "Encoding",
    "Cache",
    "Expr",
    "Encoding",
    "ExprBody",
    "JS",
    "Modify",
    "Telemetry",
    "TelemetryInner",
    "TelemetryExporter",
    "StdoutExporter",
    "OtlpExporter",
    "PrometheusFormat",
    "PrometheusExporter",
    "Apollo",
    "Cors",
];