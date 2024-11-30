use tailcall_valid::{Valid, Validator};

use super::{compile_call, compile_expr, compile_graphql, compile_grpc, compile_http, compile_js};
use crate::core::blueprint::{BlueprintError, FieldDefinition};
use crate::core::config::{self, ConfigModule, Field, GraphQLOperationType, Resolver};
use crate::core::directive::DirectiveCodec;
use crate::core::ir::model::IR;
use crate::core::try_fold::TryFold;

pub struct CompileResolver<'a> {
    pub config_module: &'a ConfigModule,
    pub field: &'a Field,
    pub operation_type: &'a GraphQLOperationType,
    pub object_name: &'a str,
}

pub fn compile_resolver(
    inputs: &CompileResolver,
    resolver: &Resolver,
) -> Valid<Option<IR>, BlueprintError> {
    let CompileResolver { config_module, field, operation_type, object_name } = inputs;

    match resolver {
        Resolver::Http(http) => {
            compile_http(config_module, http, field).trace(config::Http::trace_name().as_str())
        }
        Resolver::Grpc(grpc) => compile_grpc(super::CompileGrpc {
            config_module,
            operation_type,
            field,
            grpc,
            validate_with_schema: true,
        })
        .trace(config::Grpc::trace_name().as_str()),
        Resolver::Graphql(graphql) => {
            compile_graphql(config_module, operation_type, field.type_of.name(), graphql)
                .trace(config::GraphQL::trace_name().as_str())
        }
        Resolver::Call(call) => compile_call(config_module, call, operation_type, object_name)
            .trace(config::Call::trace_name().as_str()),
        Resolver::Js(js) => {
            compile_js(super::CompileJs { js, script: &config_module.extensions().script })
                .trace(config::JS::trace_name().as_str())
        }
        Resolver::Expr(expr) => {
            compile_expr(super::CompileExpr { config_module, field, expr, validate: true })
                .trace(config::Expr::trace_name().as_str())
        }
        Resolver::ApolloFederation(_) => {
            // ignore the Federation resolvers since they have special meaning
            // and should be executed only after the other config processing
            return Valid::succeed(None);
        }
    }
    .map(Some)
}

pub fn update_resolver<'a>(
    operation_type: &'a GraphQLOperationType,
    object_name: &'a str,
) -> TryFold<
    'a,
    (&'a ConfigModule, &'a Field, &'a config::Type, &'a str),
    FieldDefinition,
    BlueprintError,
> {
    TryFold::<(&ConfigModule, &Field, &config::Type, &str), FieldDefinition, BlueprintError>::new(
        |(config_module, field, type_of, _), b_field| {
            let inputs = CompileResolver { config_module, field, operation_type, object_name };

            Valid::from_iter(field.resolvers.iter(), |resolver| {
                compile_resolver(&inputs, resolver)
            })
            .map(|mut resolvers| match resolvers.len() {
                0 => None,
                1 => resolvers.pop().unwrap(),
                _ => Some(IR::Merge(resolvers.into_iter().flatten().collect())),
            })
            .map(|resolver| b_field.resolver(resolver))
            .and_then(|b_field| {
                b_field
                    // TODO: there are `validate_field` for field, but not for types
                    // when we use federations's entities
                    .validate_field(type_of, config_module)
                    .map_to(b_field)
            })
        },
    )
}
