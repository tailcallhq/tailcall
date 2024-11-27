use tailcall_valid::Valid;
use crate::core::blueprint::Definition;
use crate::core::blueprint::from_service_document::from_service_document::BlueprintMetadata;

impl BlueprintMetadata {
    pub(super) fn populate_resolvers(&self, defs: Vec<Definition>) -> Valid<Vec<Definition>, super::Error> {
        todo!()
    }
}