use std::collections::HashMap;
use std::fmt::Write;

use async_graphql::parser::types::{SchemaDefinition, ServiceDocument, TypeSystemDefinition};

use super::{compile_call, compile_expr, compile_graphql, compile_grpc, compile_http, compile_js};
use crate::core::blueprint::FieldDefinition;
use crate::core::config::{
    ApolloFederation, Config, ConfigModule, EntityResolver, Field, GraphQLOperationType, Resolver,
};
use crate::core::ir::model::IR;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, Validator};
use crate::core::{config, Type};

pub struct CompileEntityResolver<'a> {
    config_module: &'a ConfigModule,
    entity_resolver: &'a EntityResolver,
    operation_type: &'a GraphQLOperationType,
}

pub fn compile_entity_resolver(inputs: CompileEntityResolver<'_>) -> Valid<IR, String> {
    let CompileEntityResolver { config_module, entity_resolver, operation_type } = inputs;
    let mut resolver_by_type = HashMap::new();

    Valid::from_iter(
        entity_resolver.resolver_by_type.iter(),
        |(type_name, resolver)| {
            // Fake field that is required for validation in some cases
            // TODO: should be a proper way to run the validation both
            // on types and fields
            let field = &Field { type_of: Type::from(type_name.clone()), ..Default::default() };

            // TODO: make this code reusable in other operators like call
            let ir = match resolver {
                // TODO: there are `validate_field` for field, but not for types
                // implement validation as shared entity and use it for types
                Resolver::Http(http) => compile_http(
                    config_module,
                    http,
                    // inner resolver should resolve only single instance of type, not a list
                    false,
                ),
                Resolver::Grpc(grpc) => compile_grpc(super::CompileGrpc {
                    config_module,
                    operation_type,
                    field,
                    grpc,
                    validate_with_schema: true,
                }),
                Resolver::Graphql(graphql) => {
                    compile_graphql(config_module, operation_type, type_name, graphql)
                }
                Resolver::Call(call) => {
                    compile_call(config_module, call, operation_type, type_name)
                }
                Resolver::Js(js) => {
                    compile_js(super::CompileJs { js, script: &config_module.extensions().script })
                }
                Resolver::Expr(expr) => {
                    compile_expr(super::CompileExpr { config_module, field, expr, validate: true })
                }
                Resolver::ApolloFederation(federation) => match federation {
                    ApolloFederation::EntityResolver(entity_resolver) => {
                        compile_entity_resolver(CompileEntityResolver { entity_resolver, ..inputs })
                    }
                    ApolloFederation::Service => Valid::fail(
                        "Apollo federation resolvers can't be a part of entity resolver"
                            .to_string(),
                    ),
                },
            };

            ir.map(|ir| {
                resolver_by_type.insert(type_name.to_owned(), ir);
            })
        },
    )
    .map_to(IR::Entity(resolver_by_type))
}

pub fn compile_service(config: &ConfigModule) -> Valid<IR, String> {
    let mut service_doc = crate::core::document::print(filter_conflicting_directives(
        config.config().into(),
    ));

    let additional_schema = crate::core::document::print(filter_conflicting_directives(
        Config::graphql_schema(),
    ));
    
    let federation_v2_extension = r#"
        extend schema @link(
            url: "https://specs.apollo.dev/federation/v2.3",
            import: ["@key", "@tag", "@shareable", "@inaccessible", "@override", "@external", "@provides", "@requires", "@composeDirective", "@interfaceObject"]
        )
    "#;

    writeln!(service_doc, "{}\n{}", additional_schema, federation_v2_extension).ok();

    Valid::succeed(IR::Service(service_doc))
}

fn filter_conflicting_directives(sd: ServiceDocument) -> ServiceDocument {
    fn filter_directive(directive_name: &str) -> bool {
        directive_name != "link"
    }

    fn filter_map(def: TypeSystemDefinition) -> Option<TypeSystemDefinition> {
        match def {
            TypeSystemDefinition::Schema(schema) => {
                Some(TypeSystemDefinition::Schema(schema.map(|schema| {
                    SchemaDefinition {
                        directives: schema
                            .directives
                            .into_iter()
                            .filter(|d| filter_directive(d.node.name.node.as_str()))
                            .collect(),
                        ..schema
                    }
                })))
            }
            TypeSystemDefinition::Directive(directive) => {
                if filter_directive(directive.node.name.node.as_str()) {
                    Some(TypeSystemDefinition::Directive(directive))
                } else {
                    None
                }
            }
            ty => Some(ty),
        }
    }

    ServiceDocument {
        definitions: sd.definitions.into_iter().filter_map(filter_map).collect(),
    }
}

pub fn update_apollo_federation<'a>(
    operation_type: &'a GraphQLOperationType,
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a config::Type, &'a str), FieldDefinition, String>
{
    TryFold::<(&ConfigModule, &Field, &config::Type, &'a str), FieldDefinition, String>::new(
        |(config_module, field, _, _), b_field| {
            let Some(Resolver::ApolloFederation(federation)) = &field.resolver else {
                return Valid::succeed(b_field);
            };

            match federation {
                ApolloFederation::EntityResolver(entity_resolver) => {
                    compile_entity_resolver(CompileEntityResolver {
                        config_module,
                        entity_resolver,
                        operation_type,
                    })
                }
                ApolloFederation::Service => compile_service(config_module),
            }
            .map(|resolver| b_field.resolver(Some(resolver)))
        },
    )
}
