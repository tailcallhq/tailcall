use async_graphql::parser::types::{InputObjectType, TypeDefinition};
use async_graphql::Positioned;
use tailcall_valid::Valid;
use crate::core::blueprint::Definition;
use crate::core::blueprint::from_service_document::from_service_document::BlueprintMetadata;

impl BlueprintMetadata {
    pub(super) fn to_input_object_ty(
        &self,
        inp: &InputObjectType,
        type_definition: &Positioned<TypeDefinition>,
    ) -> Valid<Definition, super::Error> {
        todo!()
    }
}