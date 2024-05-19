// Traits and implementations for converting entities to GraphQL format.
use async_graphql::ServiceDocument;
use crate::entity::Entity;

pub trait ToGraphql {
    fn to_graphql(&self, doc: &mut ServiceDocument);
}

impl ToGraphql for Entity {
    // Converts an entity to GraphQL format and adds it to the document.
    fn to_graphql(&self, doc: &mut ServiceDocument) {
        match self {
            Entity::Schema => {
                doc.add_schema("SCHEMA");
            }
            Entity::Object => {
                doc.add_object("OBJECT");
            }
            Entity::FieldDefinition => {
                doc.add_field_definition("FIELD_DEFINITION");
            }
        }
    }
}

impl ToGraphql for Vec<Entity> {
    fn to_graphql(&self, doc: &mut ServiceDocument) {
        for entry in self {
            entry.to_graphql(doc);
        }
    }
}