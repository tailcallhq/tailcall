use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dynamic::{self, FieldFuture, FieldValue, SchemaBuilder};
use async_graphql::ErrorExtensions;
use async_graphql_value::ConstValue;
use futures_util::TryFutureExt;
use tracing::Instrument;

use crate::core::blueprint::{Blueprint, Definition, Type};
use crate::core::http::RequestContext;
use crate::core::ir::{Eval, EvaluationContext, ResolverContext};
use crate::core::json::JsonSchema;
use crate::core::scalar::CUSTOM_SCALARS;
use crate::core::valid::Validator;

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

fn to_json_schema(blueprint: &Blueprint, ty_: &Type) -> JsonSchema {
    let type_of = ty_.name();
    let list = ty_.is_list();
    let required = !ty_.is_nullable();
    let def = blueprint
        .definitions
        .iter()
        .find(|def| def.name() == type_of);

    let schema = if let Some(def) = def {
        match def {
            Definition::Interface(_) => todo!(),
            Definition::Object(obj) => {
                let mut schema_fields = HashMap::new();
                for field in obj.fields.iter() {
                    schema_fields.insert(
                        field.name.clone(),
                        to_json_schema(blueprint, &field.of_type),
                    );
                }
                JsonSchema::Obj(schema_fields)
            }
            Definition::InputObject(_) => todo!(),
            Definition::Scalar(_) => todo!(),
            Definition::Enum(type_enum) => JsonSchema::Enum(
                type_enum
                    .enum_values
                    .iter()
                    .map(|var| var.name.clone())
                    .collect(),
            ),
            Definition::Union(_) => todo!(),
        }
    } else {
        match type_of {
            "String" => JsonSchema::Str,
            "Int" => JsonSchema::Num,
            "Boolean" => JsonSchema::Bool,
            "Empty" => JsonSchema::Empty,
            "JSON" => JsonSchema::Any,
            _ => JsonSchema::Any,
        }
    };

    if !required {
        if list {
            JsonSchema::Opt(Box::new(JsonSchema::Arr(Box::new(schema))))
        } else {
            JsonSchema::Opt(Box::new(schema))
        }
    } else if list {
        JsonSchema::Arr(Box::new(schema))
    } else {
        schema
    }
}

fn to_field_value(
    schemas: &Option<Vec<(String, JsonSchema)>>,
    value: async_graphql::Value,
) -> FieldValue<'static> {
    match schemas {
        Some(schemas) => {
            for (name, schema) in schemas {
                if schema.validate(&value).is_succeed() {
                    return FieldValue::from(value).with_type(name.clone());
                }
            }

            FieldValue::from(value)
        }
        None => FieldValue::from(value),
    }
}

fn to_type(blueprint: &Blueprint, def: &Definition) -> dynamic::Type {
    match def {
        Definition::Object(def) => {
            let mut object = dynamic::Object::new(def.name.clone());
            for field in def.fields.iter() {
                let dyn_field = Arc::new(field.clone());
                let type_ref = to_type_ref(&field.of_type);
                let field_name = &field.name.clone();

                let type_name = type_ref.type_name();
                let union_schemas = blueprint
                    .definitions
                    .iter()
                    .find_map(|def| match def {
                        Definition::Union(union_) if &union_.name == type_name => Some(union_),
                        _ => None,
                    })
                    .map(|def| {
                        def.types
                            .iter()
                            .map(|type_name| {
                                // not a list and non_null because we handle list later
                                // if FieldFuture and we want to check non_null entries actually
                                let type_ =
                                    Type::NamedType { name: type_name.clone(), non_null: true };

                                (type_name.clone(), to_json_schema(blueprint, &type_))
                            })
                            .collect::<Vec<_>>()
                    });
                let union_schemas = Arc::new(union_schemas);

                let mut dyn_schema_field = dynamic::Field::new(
                    field_name,
                    type_ref.clone(),
                    move |ctx| {
                        let req_ctx = ctx.ctx.data::<Arc<RequestContext>>().unwrap();
                        let field_name = &dyn_field.name;

                        match &dyn_field.resolver {
                            None => {
                                let ctx: ResolverContext = ctx.into();
                                let ctx = EvaluationContext::new(req_ctx, &ctx);

                                let value = ctx
                                    .path_value(&[field_name])
                                    .map(|a| to_field_value(&union_schemas, a.into_owned()));

                                FieldFuture::Value(value)
                            }
                            Some(expr) => {
                                let span = tracing::info_span!(
                                    "field_resolver",
                                    otel.name = ctx.path_node.map(|p| p.to_string()).unwrap_or(field_name.clone()), graphql.returnType = %type_ref
                                );

                                let expr = expr.to_owned();
                                let union_schemas = union_schemas.clone();
                                FieldFuture::new(
                                    async move {
                                        let ctx: ResolverContext = ctx.into();
                                        let ctx = EvaluationContext::new(req_ctx, &ctx);

                                        let const_value =
                                            expr.eval(ctx).await.map_err(|err| err.extend())?;
                                        let p = match const_value {
                                            ConstValue::List(a) => Some(FieldValue::list(
                                                a.into_iter()
                                                    .map(|a| to_field_value(&union_schemas, a)),
                                            )),
                                            ConstValue::Null => FieldValue::NONE,
                                            a => Some(to_field_value(&union_schemas, a)),
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

impl From<&Blueprint> for SchemaBuilder {
    fn from(blueprint: &Blueprint) -> Self {
        let query = blueprint.query();
        let mutation = blueprint.mutation();
        let mut schema = dynamic::Schema::build(query.as_str(), mutation.as_deref(), None);

        for (k, v) in CUSTOM_SCALARS.iter() {
            schema = schema.register(dynamic::Type::Scalar(
                dynamic::Scalar::new(k.clone()).validator(v.validate()),
            ));
        }

        for def in blueprint.definitions.iter() {
            schema = schema.register(to_type(blueprint, def));
        }

        schema
    }
}
