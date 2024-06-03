use super::{
    AddField, Arg, Cache, Call, Enum, Expr, Field, GraphQL, Grpc, Http, Inline, Link, LinkType,
    Modify, Omit, Protected, Step, Tag, Telemetry, TelemetryExporter, Type, Union, JS,
};

pub trait PositionedConfig {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize));
}

impl PositionedConfig for Type {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Tag {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Cache {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Protected {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Omit {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Field {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for JS {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Modify {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Inline {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Arg {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Union {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Enum {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Http {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Call {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Step {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Grpc {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for GraphQL {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Expr {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for AddField {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Telemetry {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for TelemetryExporter {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for Link {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}

impl PositionedConfig for LinkType {
    fn set_field_position(&mut self, _field: &str, _position: (usize, usize)) {}
}
