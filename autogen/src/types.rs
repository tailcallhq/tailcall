use schemars::schema::ObjectValidation;
use std::io::Write;

use lazy_static::lazy_static;

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
        ("telemetry", vec![Entity::FieldDefinition], false),
        ("omit", vec![Entity::FieldDefinition], false),
        ("groupBy", vec![Entity::FieldDefinition], false),
        ("const", vec![Entity::FieldDefinition], false),
        ("protected", vec![Entity::FieldDefinition], false),
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
    "Const",
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

#[derive(Clone, Copy)]
pub enum Entity {
    Schema,
    Object,
    FieldDefinition,
}

pub trait ToGraphql {
    fn to_graphql(&self, f: &mut impl Write) -> std::io::Result<()>;
}

impl ToGraphql for Entity {
    fn to_graphql(&self, f: &mut impl Write) -> std::io::Result<()> {
        match self {
            Entity::Schema => {
                write!(f, "SCHEMA")
            }
            Entity::Object => {
                write!(f, "OBJECT")
            }
            Entity::FieldDefinition => {
                write!(f, "FIELD_DEFINITION")
            }
        }
    }
}

impl ToGraphql for Vec<Entity> {
    fn to_graphql(&self, f: &mut impl Write) -> std::io::Result<()> {
        let mut iter = self.iter();

        let Some(first) = iter.next() else {
            return Ok(());
        };

        write!(f, " on ")?;
        first.to_graphql(f)?;

        for entry in iter {
            write!(f, " | ")?;
            entry.to_graphql(f)?;
        }

        write!(f, "\n\n")
    }
}

pub struct LineBreaker<'a> {
    string: &'a str,
    break_at: usize,
    index: usize,
}

impl<'a> LineBreaker<'a> {
    pub fn new(string: &'a str, break_at: usize) -> Self {
        LineBreaker { string, break_at, index: 0 }
    }
}

impl<'a> Iterator for LineBreaker<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.string.len() {
            return None;
        }

        let end_index = self
            .string
            .chars()
            .skip(self.index + self.break_at)
            .enumerate()
            .find(|(_, ch)| ch.is_whitespace())
            .map(|(index, _)| self.index + self.break_at + index + 1)
            .unwrap_or(self.string.len());

        let start_index = self.index;
        self.index = end_index;

        Some(&self.string[start_index..end_index])
    }
}

#[derive(Debug)]
pub enum ExtraTypes {
    Schema,
    ObjectValidation(ObjectValidation),
}
