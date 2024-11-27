use std::collections::HashMap;
use std::fmt::Write;

use async_graphql::parser::types::ServiceDocument;
use tailcall_valid::{Valid, Validator};

use super::{compile_resolver, CompileResolver};
use crate::core::blueprint::{Blueprint, BlueprintError, Definition, TryFoldConfig};
use crate::core::config::{
    ApolloFederation, ConfigModule, EntityResolver, Field, GraphQLOperationType, Resolver,
};
use crate::core::ir::model::IR;
use crate::core::Type;

pub struct CompileEntityResolver<'a> {
    pub config_module: &'a ConfigModule,
    pub entity_resolver: &'a EntityResolver,
}

pub fn compile_entity_resolver(inputs: CompileEntityResolver<'_>) -> Valid<IR, BlueprintError> {
    let CompileEntityResolver { config_module, entity_resolver } = inputs;
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
                Resolver::ApolloFederation(federation) => match federation {
                    ApolloFederation::EntityResolver(entity_resolver) => {
                        compile_entity_resolver(CompileEntityResolver { entity_resolver, ..inputs })
                    }
                    ApolloFederation::Service => {
                        Valid::fail(BlueprintError::ApolloFederationResolversNoPartOfEntityResolver)
                    }
                },
                resolver => {
                    let inputs = CompileResolver {
                        config_module,
                        field,
                        operation_type: &GraphQLOperationType::Query,
                        object_name: type_name,
                    };

                    compile_resolver(&inputs, resolver).and_then(|resolver| {
                        Valid::from_option(resolver, BlueprintError::NoResolverFoundInSchema)
                    })
                }
            };

            ir.map(|ir| {
                resolver_by_type.insert(type_name.to_owned(), ir);
            })
        },
    )
    .map_to(IR::Entity(resolver_by_type))
}

pub fn compile_service(mut sdl: String) -> Valid<IR, BlueprintError> {
    writeln!(sdl).ok();

    // Mark subgraph as Apollo federation v2 compatible according to [docs](https://www.apollographql.com/docs/apollo-server/using-federation/apollo-subgraph-setup/#2-opt-in-to-federation-2)
    // (borrowed from async_graphql)
    writeln!(sdl, "extend schema @link(").ok();
    writeln!(sdl, "\turl: \"https://specs.apollo.dev/federation/v2.3\",").ok();
    writeln!(sdl, "\timport: [\"@key\", \"@tag\", \"@shareable\", \"@inaccessible\", \"@override\", \"@external\", \"@provides\", \"@requires\", \"@composeDirective\", \"@interfaceObject\"]").ok();
    writeln!(sdl, ")").ok();

    Valid::succeed(IR::Service(sdl))
}

pub fn update_federation<'a>() -> TryFoldConfig<'a, Blueprint> {
    TryFoldConfig::<Blueprint>::new(|config_module, mut blueprint| {
        if !config_module.server.get_enable_federation() {
            return Valid::succeed(blueprint);
        }

        // first convert to sdl with definitions in place
        let mut sdl = crate::core::document::print(ServiceDocument::from(&blueprint));
        // take definitions to update it below
        let definitions = std::mem::take(&mut blueprint.definitions);
        let query_name = blueprint.query();

        Valid::from_iter(definitions, |def| {
            if def.name() != query_name {
                return Valid::succeed(def);
            }

            let Definition::Object(mut obj) = def else {
                return Valid::fail(BlueprintError::QueryTypeNotObject);
            };

            let Some(config_type) = config_module.types.get(&query_name) else {
                return Valid::fail(BlueprintError::TypeNotFoundInConfig(query_name.clone()));
            };

            Valid::from_iter(obj.fields.iter_mut(), |b_field| {
                let b_field = std::mem::take(b_field);
                let name = &b_field.name;
                Valid::from_option(
                    config_type.fields.get(name),
                    BlueprintError::FieldNotFoundInType(name.clone()),
                )
                .and_then(|field| {
                    let federation = field
                        .resolvers
                        .iter()
                        .find(|&resolver| matches!(resolver, Resolver::ApolloFederation(_)));

                    let Some(Resolver::ApolloFederation(federation)) = federation else {
                        return Valid::succeed(b_field);
                    };

                    match federation {
                        ApolloFederation::EntityResolver(entity_resolver) => {
                            compile_entity_resolver(CompileEntityResolver {
                                config_module,
                                entity_resolver,
                            })
                        }
                        ApolloFederation::Service => compile_service(std::mem::take(&mut sdl)),
                    }
                    .map(|resolver| b_field.resolver(Some(resolver)))
                })
            })
            .map(|fields| {
                obj.fields = fields;

                Definition::Object(obj)
            })
        })
        .map(|definitions| blueprint.definitions(definitions))
    })
}
