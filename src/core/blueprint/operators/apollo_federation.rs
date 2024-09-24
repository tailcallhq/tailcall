use std::collections::HashMap;
use std::fmt::Write;

use async_graphql::parser::types::ServiceDocument;

use super::{compile_call, compile_expr, compile_graphql, compile_grpc, compile_http, compile_js};
use crate::core::blueprint::FieldDefinition;
use crate::core::config::{
    ApolloFederation, Config, ConfigModule, EntityResolver, Field, GraphQLOperationType, Resolver,
};
use crate::core::ir::model::IR;
use crate::core::sdl::SdlPrinter;
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
    let sdl_printer = SdlPrinter { federation_compatibility: true };
    let mut sdl = sdl_printer.print(ServiceDocument::from(config.config()));

    writeln!(sdl).ok();
    // Add tailcall specific definitions to the sdl output
    writeln!(sdl, "{}", sdl_printer.print(Config::graphql_schema())).ok();
    writeln!(sdl).ok();
    // Mark subgraph as Apollo federation v2 compatible according to [docs](https://www.apollographql.com/docs/apollo-server/using-federation/apollo-subgraph-setup/#2-opt-in-to-federation-2)
    // (borrowed from async_graphql)
    writeln!(sdl, "extend schema @link(").ok();
    writeln!(sdl, "\turl: \"https://specs.apollo.dev/federation/v2.3\",").ok();
    writeln!(sdl, "\timport: [\"@key\", \"@tag\", \"@shareable\", \"@inaccessible\", \"@override\", \"@external\", \"@provides\", \"@requires\", \"@composeDirective\", \"@interfaceObject\"]").ok();
    writeln!(sdl, ")").ok();

    Valid::succeed(IR::Service(sdl))
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
