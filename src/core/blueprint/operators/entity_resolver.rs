use std::collections::HashMap;

use super::{compile_call, compile_expr, compile_graphql, compile_grpc, compile_http, compile_js};
use crate::core::blueprint::FieldDefinition;
use crate::core::config::{
    ConfigModule, EntityResolver, Field, GraphQLOperationType, Resolver, Type,
};
use crate::core::ir::model::IR;
use crate::core::try_fold::TryFold;
use crate::core::valid::{Valid, Validator};

pub struct CompileEntityResolver<'a> {
    config_module: &'a ConfigModule,
    field: &'a Field,
    entity_resolver: &'a EntityResolver,
    operation_type: &'a GraphQLOperationType,
    object_name: &'a str,
}

pub fn compile_entity_resolver(inputs: CompileEntityResolver<'_>) -> Valid<IR, String> {
    let CompileEntityResolver {
        config_module,
        field,
        entity_resolver,
        operation_type,
        object_name,
    } = inputs;
    let mut resolver_by_type = HashMap::new();

    Valid::from_iter(
        entity_resolver.resolver_by_type.iter(),
        |(type_name, resolver)| {
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
                    compile_call(config_module, call, operation_type, object_name)
                }
                Resolver::Js(js) => {
                    compile_js(super::CompileJs { js, script: &config_module.extensions().script })
                }
                Resolver::Expr(expr) => {
                    compile_expr(super::CompileExpr { config_module, field, expr, validate: true })
                }
                Resolver::EntityResolver(entity_resolver) => {
                    compile_entity_resolver(CompileEntityResolver { entity_resolver, ..inputs })
                }
            };

            ir.map(|ir| {
                resolver_by_type.insert(type_name.to_owned(), ir);
            })
        },
    )
    .map_to(IR::EntityResolver(resolver_by_type))
}

pub fn update_entity_resolver<'a>(
    operation_type: &'a GraphQLOperationType,
    object_name: &'a str,
) -> TryFold<'a, (&'a ConfigModule, &'a Field, &'a Type, &'a str), FieldDefinition, String> {
    TryFold::<(&ConfigModule, &Field, &Type, &'a str), FieldDefinition, String>::new(
        |(config_module, field, _, _), b_field| {
            let Some(Resolver::EntityResolver(entity_resolver)) = &field.resolver else {
                return Valid::succeed(b_field);
            };

            compile_entity_resolver(CompileEntityResolver {
                config_module,
                field,
                entity_resolver,
                operation_type,
                object_name,
            })
            .map(|resolver| b_field.resolver(Some(resolver)))
        },
    )
}
