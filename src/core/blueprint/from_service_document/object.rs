use async_graphql::parser::types::{FieldDefinition, InterfaceType, ObjectType, TypeDefinition};
use async_graphql::Positioned;
use async_graphql_value::Name;
use tailcall_valid::{Valid, Validator};

use crate::core::{blueprint, Type};
use crate::core::blueprint::Definition;
use crate::core::blueprint::from_service_document::{Error, helpers, pos_name_to_string};
use crate::core::blueprint::from_service_document::from_service_document::BlueprintMetadata;

pub(super) trait ObjectLike {
    fn fields(&self) -> &Vec<Positioned<FieldDefinition>>;
    fn implements(&self) -> &Vec<Positioned<Name>>;
}
impl ObjectLike for ObjectType {
    fn fields(&self) -> &Vec<Positioned<FieldDefinition>> {
        &self.fields
    }
    fn implements(&self) -> &Vec<Positioned<Name>> {
        &self.implements
    }
}
impl ObjectLike for InterfaceType {
    fn fields(&self) -> &Vec<Positioned<FieldDefinition>> {
        &self.fields
    }
    fn implements(&self) -> &Vec<Positioned<Name>> {
        &self.implements
    }
}

impl BlueprintMetadata {
    pub(super) fn to_object_ty<Obj: ObjectLike>(
        &self,
        obj: &Obj,
        type_definition: &Positioned<TypeDefinition>,
    ) -> Valid<Definition, Error> {
        Valid::from_iter(obj.fields().iter(), |f| {
            let field = &f.node;
            let field_name = field.name.node.to_string();
            let field_type = &field.ty.node;
            let field_args = field.arguments.iter().map(|arg| {
                let arg = &arg.node;
                let arg_name = arg.name.node.to_string();
                let arg_type = &arg.ty.node;
                let arg_default = arg.default_value.as_ref().map(|v| v.node.clone().into_json().ok()).flatten();
                blueprint::InputFieldDefinition {
                    name: arg_name,
                    of_type: Type::from(arg_type),
                    default_value: arg_default,
                    description: arg.description.as_ref().map(|d| d.node.to_string()),
                }
            }).collect();
            let directives = helpers::extract_directives(field.directives.iter());
            directives.and_then(|directives| {
                let field_type = blueprint::FieldDefinition {
                    name: field_name,
                    args: field_args,
                    directives,
                    description: field.description.as_ref().map(|d| d.node.to_string()),
                    resolver: None,
                    of_type: Type::from(field_type),
                    default_value: None,
                };
                Valid::succeed(field_type)
            })
        }).and_then(|fields| {
            helpers::extract_directives(type_definition.node.directives.iter()).and_then(|directives| {
                Valid::succeed(
                    blueprint::ObjectTypeDefinition {
                        name: pos_name_to_string(&type_definition.node.name),
                        directives,
                        description: type_definition.node.description.as_ref().map(|d| d.node.to_string()),
                        fields,
                        implements: obj.implements().iter().map(|i| i.node.to_string()).collect(),
                    }
                )
            })
        }).and_then(|obj| {
            Valid::succeed(Definition::Object(obj))
        })
    }
}
