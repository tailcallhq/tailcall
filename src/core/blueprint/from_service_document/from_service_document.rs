use std::slice::Iter;

use async_graphql::parser::types::{ConstDirective, ServiceDocument, TypeKind, TypeSystemDefinition};
use async_graphql::Positioned;
use tailcall_valid::{Valid, ValidationError, Validator};

use crate::core::{blueprint, Type};
use crate::core::blueprint::{Blueprint, Definition};
use crate::core::blueprint::from_service_document::schema;
use crate::core::scalar::Scalar;
use crate::core::try_fold::TryFold;

pub struct BlueprintMetadata {
    pub path: String,
    pub doc: ServiceDocument,
}

impl BlueprintMetadata {
    pub fn new(path: String, doc: ServiceDocument) -> Self {
        Self { path, doc }
    }

    pub fn to_blueprint<'a>(&self) -> Valid<Blueprint, super::Error> {
        let schema = self.to_schema().transform::<Blueprint>(
            |schema, blueprint| blueprint.schema(schema),
            |blueprint| blueprint.schema,
        );
        let definitions = self.to_definitions();

        let result = schema
            .and(definitions)
            .try_fold(&self.doc, Blueprint::default());
        result
    }

    fn to_schema<'a>(&self) -> TryFold<'a, ServiceDocument, blueprint::SchemaDefinition, super::Error> {
        TryFold::<ServiceDocument, blueprint::SchemaDefinition, super::Error>::new(|doc, schema| {
            self.schema_definition().and_then(|schema_def| {
                self.to_bp_schema_def(&schema_def)
            })
        })
    }

    fn to_definitions<'a>(&self) -> TryFold<'a, ServiceDocument, Vec<Definition>, super::Error> {
        TryFold::<ServiceDocument, Vec<Definition>, super::Error>::new(|doc, defs| {
            todo!()
        })
    }

    fn populate_defs(mut bp: Vec<Definition>, doc: &ServiceDocument) -> Valid<Vec<Definition>, super::Error> {
        Valid::succeed(&mut bp).and_then(|bp| {
            Valid::from_iter(doc.definitions.iter(), |def| {
                match def {
                    TypeSystemDefinition::Schema(schema) => {
                        Valid::succeed(())
                    }
                    TypeSystemDefinition::Type(ty) => {
                        let ty_def = &ty.node;
                        let ty_name = ty_def.name.to_string();
                        let ty = &ty_def.kind;
                        match ty {
                            TypeKind::Scalar => {
                                if Scalar::is_predefined(&ty_name) {
                                    Valid::fail(format!("Scalar type `{}` is predefined", ty_name))
                                } else {
                                    Valid::succeed(ty_name)
                                }.and_then(|sclar| {
                                    super::helpers::extract_directives(ty_def.directives.iter()).and_then(|directives| {
                                        bp.push(blueprint::Definition::Scalar(blueprint::ScalarTypeDefinition {
                                            scalar: Scalar::find(&sclar).unwrap_or(&Scalar::Empty).clone(),
                                            name: sclar.to_string(),
                                            directives,
                                            description: ty_def.description.as_ref().map(|d| d.node.to_string()),
                                        }));
                                        Valid::succeed(())
                                    })
                                })
                            }
                            TypeKind::Object(obj) => {
                                Valid::from_iter(obj.fields.iter(), |field| {
                                    let field = &field.node;
                                    let field_name = field.name.node.to_string();
                                    let field_type = &field.ty.node;
                                    Valid::from_iter(field.arguments.iter(), |arg| {
                                        let arg = &arg.node;
                                        let arg_name = arg.name.node.to_string();
                                        let arg_type = &arg.ty.node;
                                        let arg_default = arg.default_value.as_ref().map(|v| v.node.clone().into_json().ok()).flatten();
                                        let arg_def = blueprint::InputFieldDefinition {
                                            name: arg_name,
                                            of_type: Type::from(arg_type),
                                            default_value: arg_default,
                                            description: arg.description.as_ref().map(|d| d.node.to_string()),
                                        };

                                        Valid::succeed(arg_def)
                                    }).and_then(|args| {
                                        super::helpers::extract_directives(field.directives.iter()).map(|directives| (args, directives))
                                    }).and_then(|(args, directives)| {
                                        let field_type = blueprint::FieldDefinition {
                                            name: field_name,
                                            args,
                                            directives,
                                            description: field.description.as_ref().map(|d| d.node.to_string()),
                                            resolver: None,
                                            of_type: Type::from(field_type),
                                            default_value: None,
                                        };
                                        Valid::succeed(field_type)
                                    })
                                }).and_then(|fields| {
                                    super::helpers::extract_directives(ty_def.directives.iter())
                                        .and_then(|directives| {
                                            let obj_def = blueprint::ObjectTypeDefinition {
                                                name: ty_name,
                                                directives,
                                                description: ty_def.description.as_ref().map(|d| d.node.to_string()),
                                                fields,
                                                implements: obj.implements.iter().map(|i| i.node.to_string()).collect(),
                                            };
                                            bp.push(blueprint::Definition::Object(obj_def));
                                            Valid::succeed(())
                                        })
                                })
                            }
                            TypeKind::Interface(interface) => {
                                Valid::from_iter(interface.fields.iter(), |field| {
                                    let field = &field.node;
                                    let field_name = field.name.node.to_string();
                                    let field_type = &field.ty.node;
                                    super::helpers::extract_directives(field.directives.iter()).and_then(|directives| {
                                        Valid::from_iter(field.arguments.iter(), |arg| {
                                            let arg = &arg.node;
                                            let arg_name = arg.name.node.to_string();
                                            let arg_type = &arg.ty.node;
                                            let arg_default = arg.default_value.as_ref().map(|v| v.node.clone().into_json().ok()).flatten();
                                            let arg_def = blueprint::InputFieldDefinition {
                                                name: arg_name,
                                                of_type: Type::from(arg_type),
                                                default_value: arg_default,
                                                description: arg.description.as_ref().map(|d| d.node.to_string()),
                                            };

                                            Valid::succeed(arg_def)
                                        }).and_then(|args| {
                                            let field_type = blueprint::FieldDefinition {
                                                name: field_name,
                                                args,
                                                directives,
                                                description: field.description.as_ref().map(|d| d.node.to_string()),
                                                resolver: None,
                                                of_type: Type::from(field_type),
                                                default_value: None,
                                            };
                                            Valid::succeed(field_type)
                                        })
                                    })
                                }).and_then(|fields| {
                                    super::helpers::extract_directives(ty_def.directives.iter()).and_then(|directives| {
                                        let obj_def = blueprint::ObjectTypeDefinition {
                                            name: ty_name,
                                            directives,
                                            description: ty_def.description.as_ref().map(|d| d.node.to_string()),
                                            fields,
                                            implements: interface.implements.iter().map(|i| i.node.to_string()).collect(),
                                        };
                                        bp.push(blueprint::Definition::Object(obj_def));
                                        Valid::succeed(())
                                    })
                                })
                            }
                            TypeKind::Union(union) => {
                                super::helpers::extract_directives(ty_def.directives.iter()).and_then(|directives| {
                                    let union = blueprint::UnionTypeDefinition {
                                        name: ty_name,
                                        directives,
                                        description: ty_def.description.as_ref().map(|d| d.node.to_string()),
                                        types: union.members.iter().map(|t| t.node.to_string()).collect(),
                                    };
                                    Valid::succeed(union)
                                }).and_then(|union| {
                                    bp.push(blueprint::Definition::Union(union));
                                    Valid::succeed(())
                                })
                            }
                            TypeKind::Enum(enum_) => {
                                Valid::from_iter(enum_.values.iter(), |value| {
                                    let value = &value.node;
                                    super::helpers::extract_directives(value.directives.iter()).map(|directives| {
                                        let value = blueprint::EnumValueDefinition {
                                            name: ty_name.to_string(),
                                            directives,
                                            description: value.description.as_ref().map(|d| d.node.to_string()),
                                            // TODO: alias
                                            alias: Default::default(),
                                        };
                                        value
                                    })
                                }).and_then(|values| {
                                    super::helpers::extract_directives(ty_def.directives.iter()).and_then(|directives| {
                                        let enum_def = blueprint::EnumTypeDefinition {
                                            name: ty_name,
                                            directives,
                                            description: ty_def.description.as_ref().map(|d| d.node.to_string()),
                                            enum_values: values,
                                        };
                                        bp.push(blueprint::Definition::Enum(enum_def));
                                        Valid::succeed(())
                                    })
                                })
                            }
                            TypeKind::InputObject(input_object) => {
                                Valid::from_iter(input_object.fields.iter(), |field| {
                                    let field = &field.node;
                                    let field_name = field.name.node.to_string();
                                    let field_type = &field.ty.node;
                                    let field_default = field.default_value.as_ref().map(|v| v.node.clone().into_json().ok()).flatten();
                                    let field = blueprint::InputFieldDefinition {
                                        name: field_name,
                                        of_type: Type::from(field_type),
                                        default_value: field_default,
                                        description: field.description.as_ref().map(|d| d.node.to_string()),
                                    };
                                    Valid::succeed(field)
                                }).and_then(|fields| {
                                    super::helpers::extract_directives(ty_def.directives.iter()).and_then(|directives| {
                                        let input_object = blueprint::InputObjectTypeDefinition {
                                            name: ty_name,
                                            directives,
                                            description: ty_def.description.as_ref().map(|d| d.node.to_string()),
                                            fields,
                                        };
                                        bp.push(blueprint::Definition::InputObject(input_object));
                                        Valid::succeed(())
                                    })
                                })
                            }
                        }
                    }
                    TypeSystemDefinition::Directive(directive) => {
                        Valid::succeed(())
                    }
                }
            })
        }).and_then(|_| Valid::succeed(bp))
    }
}

#[cfg(test)]
mod tests {
    use tailcall_valid::Validator;
    use crate::core::blueprint::from_service_document::from_service_document::BlueprintMetadata;

    #[test]
    fn test_from_bp() {
        // Test code here
        let path = format!("{}/examples/hello.graphql", env!("CARGO_MANIFEST_DIR"));
        println!("{}", path);
        let doc = async_graphql::parser::parse_schema(std::fs::read_to_string(&path).unwrap()).unwrap();
        let bp = BlueprintMetadata::new(path, doc).to_blueprint().to_result().unwrap();
        println!("{:#?}", bp);
    }
}