use std::borrow::Cow;
use std::sync::Arc;

use async_graphql::dynamic::{self, FieldFuture, FieldValue, SchemaBuilder};
use async_graphql::extensions::ApolloTracing;
use async_graphql::{ErrorExtensions, ValidationMode};
use async_graphql_value::ConstValue;
use futures_util::TryFutureExt;
use tracing::Instrument;

use crate::blueprint::{Blueprint, Definition, GlobalTimeout, SchemaModifiers, Type};
use crate::http::RequestContext;
use crate::lambda::{Eval, EvaluationContext, ResolverContext};
use crate::scalar::CUSTOM_SCALARS;

fn to_type_ref(type_of: &Type) -> dynamic::TypeRef {
    match type_of {
        Type::NamedType { name, non_null } => {
            if *non_null {
                dynamic::TypeRef::NonNull(Box::from(dynamic::TypeRef::Named(Cow::Owned(
                    name.clone(),
                ))))
            } else {
                dynamic::TypeRef::Named(Cow::Owned(name.clone()))
            }
        }
        Type::ListType { of_type, non_null } => {
            let inner = Box::new(to_type_ref(of_type));
            if *non_null {
                dynamic::TypeRef::NonNull(Box::from(dynamic::TypeRef::List(inner)))
            } else {
                dynamic::TypeRef::List(inner)
            }
        }
    }
}

fn to_type(def: &Definition, no_resolver: bool) -> dynamic::Type {
    match def {
        Definition::Object(def) => {
            let mut object = dynamic::Object::new(def.name.clone());
            for field in def.fields.iter() {
                let field = field.clone();
                let type_ref = to_type_ref(&field.of_type);
                let field_name = &field.name.clone();
                let mut dyn_schema_field = dynamic::Field::new(
                    field_name,
                    type_ref.clone(),
                    move |ctx| {
                        let req_ctx = ctx.ctx.data::<Arc<RequestContext>>().unwrap();
                        let field_name = &field.name;

                        match (&field.resolver, no_resolver) {
                            (None, _) | (Some(_), true) => {
                                let ctx: ResolverContext = ctx.into();
                                let ctx = EvaluationContext::new(req_ctx, &ctx);
                                FieldFuture::from_value(
                                    ctx.path_value(&[field_name])
                                        .map(|a| a.into_owned().to_owned()),
                                )
                            }
                            (Some(expr), false) => {
                                let span = tracing::info_span!(
                                    "field_resolver",
                                    otel.name = ctx.path_node.map(|p| p.to_string()).unwrap_or(field_name.clone()), graphql.returnType = %type_ref
                                );
                                let expr = expr.to_owned();
                                FieldFuture::new(
                                    async move {
                                        let ctx: ResolverContext = ctx.into();
                                        let ctx = EvaluationContext::new(req_ctx, &ctx);

                                        let const_value =
                                            expr.eval(ctx).await.map_err(|err| err.extend())?;
                                        let p = match const_value {
                                            ConstValue::List(a) => Some(FieldValue::list(a)),
                                            ConstValue::Null => FieldValue::NONE,
                                            a => Some(FieldValue::from(a)),
                                        };
                                        Ok(p)
                                    }
                                    .instrument(span)
                                    .inspect_err(|err| tracing::error!(?err)),
                                )
                            }
                        }
                    },
                );
                if let Some(description) = &field.description {
                    dyn_schema_field = dyn_schema_field.description(description);
                }
                for arg in field.args.iter() {
                    dyn_schema_field = dyn_schema_field.argument(dynamic::InputValue::new(
                        arg.name.clone(),
                        to_type_ref(&arg.of_type),
                    ));
                }
                object = object.field(dyn_schema_field);
            }
            for interface in def.implements.iter() {
                object = object.implement(interface.clone());
            }

            dynamic::Type::Object(object)
        }
        Definition::Interface(def) => {
            let mut interface = dynamic::Interface::new(def.name.clone());
            for field in def.fields.iter() {
                interface = interface.field(dynamic::InterfaceField::new(
                    field.name.clone(),
                    to_type_ref(&field.of_type),
                ));
            }

            dynamic::Type::Interface(interface)
        }
        Definition::InputObject(def) => {
            let mut input_object = dynamic::InputObject::new(def.name.clone());
            for field in def.fields.iter() {
                input_object = input_object.field(dynamic::InputValue::new(
                    field.name.clone(),
                    to_type_ref(&field.of_type),
                ));
            }

            dynamic::Type::InputObject(input_object)
        }
        Definition::Scalar(def) => {
            let mut scalar = dynamic::Scalar::new(def.name.clone());
            if let Some(description) = &def.description {
                scalar = scalar.description(description);
            }
            scalar = scalar.validator(def.validator);
            dynamic::Type::Scalar(scalar)
        }
        Definition::Enum(def) => {
            let mut enum_type = dynamic::Enum::new(def.name.clone());
            for value in def.enum_values.iter() {
                enum_type = enum_type.item(dynamic::EnumItem::new(value.name.clone()));
            }
            if let Some(desc) = def.description.clone() {
                enum_type = enum_type.description(desc);
            }
            dynamic::Type::Enum(enum_type)
        }
        Definition::Union(def) => {
            let mut union = dynamic::Union::new(def.name.clone());
            for type_ in def.types.iter() {
                union = union.possible_type(type_.clone());
            }
            dynamic::Type::Union(union)
        }
    }
}

pub struct SchemaGenerator<'a> {
    pub blueprint: &'a Blueprint,
    pub schema_modifiers: Option<SchemaModifiers>,
}

impl<'a> SchemaGenerator<'a> {
    pub fn new(blueprint: &'a Blueprint) -> Self {
        SchemaGenerator { blueprint, schema_modifiers: None }
    }

    pub fn with_schema_modifier(
        blueprint: &'a Blueprint,
        schema_modifiers: SchemaModifiers,
    ) -> Self {
        SchemaGenerator { blueprint, schema_modifiers: Some(schema_modifiers) }
    }

    pub fn schema_modifier(self, schema_modifiers: SchemaModifiers) -> Self {
        SchemaGenerator {
            blueprint: self.blueprint,
            schema_modifiers: Some(schema_modifiers),
        }
    }

    pub fn generate_schema(self) -> SchemaBuilder {
        let blueprint = self.blueprint;
        let schema_modifiers = self.schema_modifiers;

        let server = &blueprint.server;

        let query = blueprint.query();
        let mutation = blueprint.mutation();
        let mut schema = dynamic::Schema::build(query.as_str(), mutation.as_deref(), None);

        for (k, v) in CUSTOM_SCALARS.iter() {
            schema = schema.register(dynamic::Type::Scalar(
                dynamic::Scalar::new(k.clone()).validator(v.validate()),
            ));
        }

        let no_resolver = schema_modifiers
            .as_ref()
            .map(|s| s.no_resolver)
            .unwrap_or(false);

        for def in blueprint.definitions.iter() {
            schema = schema.register(to_type(def, no_resolver));
        }

        if let Some(schema_modifiers) = schema_modifiers {
            if server.enable_apollo_tracing {
                schema = schema.extension(ApolloTracing);
            }

            if server.global_response_timeout > 0 {
                schema = schema
                    .data(async_graphql::Value::from(server.global_response_timeout))
                    .extension(GlobalTimeout);
            }

            if server.get_enable_query_validation() || schema_modifiers.no_resolver {
                schema = schema.validation_mode(ValidationMode::Strict);
            } else {
                schema = schema.validation_mode(ValidationMode::Fast);
            }

            if !server.get_enable_introspection() || schema_modifiers.no_resolver {
                schema = schema.disable_introspection();
            }

            for extension in schema_modifiers.extensions.iter().cloned() {
                schema = schema.extension(extension);
            }
        }

        schema
    }
}
