use async_graphql::parser::types::{TypeDefinition, UnionType};
use async_graphql::Positioned;
use tailcall_valid::{Valid, Validator};
use crate::core::blueprint;
use crate::core::blueprint::Definition;
use crate::core::blueprint::from_service_document::{Error, helpers, pos_name_to_string};
use crate::core::blueprint::from_service_document::from_service_document::BlueprintMetadata;

impl BlueprintMetadata {
    pub(super) fn to_union_ty(&self, union_: &UnionType, type_definition: &Positioned<TypeDefinition>) -> Valid<Definition, Error> {
        let types = union_
            .members
            .iter()
            .map(|t| t.node.to_string())
            .collect();
        helpers::extract_directives(type_definition.node.directives.iter()).and_then(|directives| {
            Valid::succeed(
                Definition::Union(
                    blueprint::UnionTypeDefinition {
                        name: pos_name_to_string(&type_definition.node.name),
                        directives,
                        description: type_definition.node.description.as_ref().map(|d| d.node.to_string()),
                        types,
                    }
                )
            )
        })
    }
}
