use async_graphql::parser::types::{EnumType, TypeDefinition};
use async_graphql::Positioned;
use tailcall_valid::{Valid, Validator};
use crate::core::blueprint;
use crate::core::blueprint::Definition;
use crate::core::blueprint::from_service_document::{Error, helpers, pos_name_to_string};
use crate::core::blueprint::from_service_document::from_service_document::BlueprintMetadata;
use crate::core::config::Alias;
use crate::core::directive::DirectiveCodec;

impl BlueprintMetadata {
    pub(super) fn to_enum_ty(&self, enum_: &EnumType, type_definition: &Positioned<TypeDefinition>) -> Valid<Definition, Error> {
        Valid::from_iter(enum_.values.iter(), |value| {
            let name = value.node.value.node.as_str().to_owned();
            let alias = value
                .node
                .directives
                .iter()
                .find(|d| d.node.name.node.as_str() == Alias::directive_name());
            let directives = helpers::extract_directives(value.node.directives.iter());

            let description = value.node.description.as_ref().map(|d| d.node.to_string());
            directives.and_then(|directives| {
                if let Some(alias) = alias {
                    Alias::from_directive(&alias.node).map(|alias| (directives, name, alias.options, description))
                } else {
                    Valid::succeed((directives, name, Default::default(), description))
                }
            })
        }).and_then(|variants| {
            helpers::extract_directives(type_definition.node.directives.iter()).and_then(|directives| {
                let enum_def = blueprint::EnumTypeDefinition {
                    name: pos_name_to_string(&type_definition.node.name),
                    directives,
                    description: type_definition.node.description.as_ref().map(|d| d.node.to_string()),
                    enum_values: variants.into_iter().map(|(directives, name, alias, description)| {
                        blueprint::EnumValueDefinition {
                            name,
                            directives,
                            description,
                            alias,
                        }
                    }).collect(),
                };
                Valid::succeed(Definition::Enum(enum_def))
            })
        })
    }
}
