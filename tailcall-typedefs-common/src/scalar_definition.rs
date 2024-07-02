use async_graphql_parser::types::{TypeDefinition, TypeKind, TypeSystemDefinition};
use async_graphql_value::Name;
use schemars::schema::{RootSchema, SchemaObject};
use schemars::JsonSchema;

use crate::common::{get_description, pos};

pub trait ScalarDefinition {
    fn into_schemars() -> RootSchema
    where
        Self: JsonSchema,
    {
        schemars::schema_for!(Self)
    }

    fn scalar_definition() -> TypeSystemDefinition;
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
