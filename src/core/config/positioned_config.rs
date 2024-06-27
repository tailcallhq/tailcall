pub trait PositionedConfig {
    fn set_field_position(&mut self, field: &str, position: (usize, usize, &str));
}
