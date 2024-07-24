use async_graphql::parser::types::{TypeDefinition, TypeKind, TypeSystemDefinition};
use async_graphql::Name;
use schemars::schema::{RootSchema, SchemaObject};

use crate::common::{get_description, pos};
use crate::core::scalar::ScalarType;

pub trait ScalarDefinition {
    fn scalar_definition() -> TypeSystemDefinition;
}

impl ScalarType {
    pub fn scalar_definition(&self) -> TypeSystemDefinition {
        let root_schema = self.schema();
        let schema: SchemaObject = root_schema.into();
        let description = get_description(&schema);
        TypeSystemDefinition::Type(pos(TypeDefinition {
            name: pos(Name::new(&self.name())),
            kind: TypeKind::Scalar,
            description: description.map(|inner| pos(inner.clone())),
            directives: vec![],
            extend: false,
        }))
    }
}

pub fn into_scalar_definition(root_schema: RootSchema, name: &str) -> TypeSystemDefinition {
    let schema: SchemaObject = root_schema.schema;
    let description = get_description(&schema);
    TypeSystemDefinition::Type(pos(TypeDefinition {
        name: pos(Name::new(name)),
        kind: TypeKind::Scalar,
        description: description.map(|inner| pos(inner.clone())),
        directives: vec![],
        extend: false,
    }))
}
