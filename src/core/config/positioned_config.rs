/// This file will be changed in favor of the macro left here untill all types are positioned
use super::{
    Arg, Enum, Field, Inline, Link, LinkType, Modify, Omit, Protected, Tag, Telemetry,
    TelemetryExporter, Union, JS,
};

pub trait PositionedConfig {
    fn set_field_position(&mut self, field: &str, _position: (usize, usize));
}

impl PositionedConfig for Tag {
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
