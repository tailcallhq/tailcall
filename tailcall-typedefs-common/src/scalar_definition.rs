use async_graphql::parser::types::{TypeDefinition, TypeKind, TypeSystemDefinition};
use async_graphql::Name;
use schemars::schema::{Schema, SchemaObject};

use crate::common::{get_description, pos};

pub trait ScalarDefinition {
    fn scalar_definition() -> TypeSystemDefinition;
}

pub fn into_scalar_definition(root_schema: Schema, name: &str) -> TypeSystemDefinition {
    let schema: SchemaObject = root_schema.into_object();
    let description = get_description(&schema);
    TypeSystemDefinition::Type(pos(TypeDefinition {
        name: pos(Name::new(name)),
        kind: TypeKind::Scalar,
        description: description.map(|inner| pos(inner.clone())),
        directives: vec![],
        extend: false,
    }))
}
