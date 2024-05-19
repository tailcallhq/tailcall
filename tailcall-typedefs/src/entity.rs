// Defines various GraphQL entities used in schema generation.
#[derive(Clone, Copy)]
pub enum Entity {
    Schema,
    Object,
    FieldDefinition,
}