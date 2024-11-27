use async_graphql::parser::types::TypeDefinition;
use async_graphql::Positioned;
use tailcall_valid::{Valid, Validator};
use crate::core::blueprint::{Definition, ScalarTypeDefinition};
use crate::core::blueprint::from_service_document::from_service_document::BlueprintMetadata;
use crate::core::blueprint::from_service_document::{helpers, pos_name_to_string};
use crate::core::scalar::Scalar;

impl BlueprintMetadata {
    pub(super) fn to_scalar_ty(&self, type_definition: &Positioned<TypeDefinition>) -> Valid<Definition, super::Error> {
        let type_name = pos_name_to_string(&type_definition.node.name);

        if Scalar::is_predefined(&type_name) {
            Valid::fail(format!("Scalar type `{}` is predefined", type_name))
        } else {
            helpers::extract_directives(type_definition.node.directives.iter()).and_then(|directives| {
                Valid::succeed(
                    Definition::Scalar(
                        ScalarTypeDefinition {
                            scalar: Scalar::find(&type_name).unwrap_or(&Scalar::Empty).clone(),
                            name: type_name,
                            directives,
                            description: type_definition.node.description.as_ref().map(|d| d.node.to_string()),
                        }
                    )
                )
            })
        }
    }
}