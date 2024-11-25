use std::sync::Arc;

use async_graphql::dynamic::{self, FieldFuture, FieldValue, SchemaBuilder, TypeRef};
use async_graphql_value::ConstValue;
use futures_util::TryFutureExt;
use tracing::Instrument;

use crate::core::blueprint::{Blueprint, Definition};
use crate::core::http::RequestContext;
use crate::core::ir::{EvalContext, ResolverContext, TypedValue};
use crate::core::jit::graphql_error::ErrorExtensions;
use crate::core::scalar::Scalar;

/// We set the default value for an `InputValue` by reading it from the
/// blueprint and assigning it to the provided `InputValue` during the
/// generation of the `async_graphql::Schema`. The `InputValue` represents the
/// structure of arguments and their types that can be passed to a field. In
/// other GraphQL implementations, this is commonly referred to as
/// `InputValueDefinition`.
fn set_default_value(
    input_value: dynamic::InputValue,
    value: Option<serde_json::Value>,
) -> dynamic::InputValue {
    if let Some(value) = value {
        match ConstValue::from_json(value) {
            Ok(const_value) => input_value.default_value(const_value),
            Err(err) => {
                tracing::warn!("conversion from serde_json::Value to ConstValue failed for default_value with error {err:?}");
                input_value
            }
        }
    } else {
        input_value
    }
}

fn to_field_value(value: async_graphql::Value) -> FieldValue<'static> {
    match value {
        ConstValue::List(vec) => FieldValue::list(vec.into_iter().map(to_field_value)),
        value => {
            let type_name = value.get_type_name().map(|s| s.to_string());

            let field_value = FieldValue::from(value);

            if let Some(type_name) = type_name {
                field_value.with_type(type_name)
            } else {
                field_value
            }
        }
    }
}

fn to_type(def: &Definition) -> dynamic::Type {
    match def {
        Definition::Object(def) => {
            let mut object = dynamic::Object::new(def.name.clone());
            for field in def.fields.iter() {
                let field = field.clone();
                let type_ref = TypeRef::from(&field.of_type);
                let field_name = &field.name.clone();

                let mut dyn_schema_field = dynamic::Field::new(
                    field_name,
                    type_ref.clone(),
                    move |ctx| {
                        // region: HOT CODE
                        // --------------------------------------------------
                        //                HOT CODE STARTS HERE
                        // --------------------------------------------------

                        let req_ctx = ctx.ctx.data::<Arc<RequestContext>>().unwrap();
                        let field_name = &field.name;

                        match &field.resolver {
                            None => {
                                let ctx: ResolverContext = ctx.into();
                                let ctx = EvalContext::new(req_ctx, &ctx);

                                match ctx.path_value(&[field_name]).map(|a| a.into_owned()) {
                                    Some(ConstValue::Null) => FieldFuture::Value(FieldValue::NONE),
                                    a => FieldFuture::from_value(a),
                                }
                            }
                            Some(expr) => {
                                let span = tracing::info_span!(
                                    "field_resolver",
                                    otel.name = ctx.path_node.map(|p| p.to_string()).unwrap_or(field_name.clone()), graphql.returnType = %type_ref
                                );

                                let expr = expr.to_owned();
                                FieldFuture::new(
                                    async move {
                                        let ctx: ResolverContext = ctx.into();
                                        let ctx = &mut EvalContext::new(req_ctx, &ctx);

                                        let value =
                                            expr.eval(ctx).await.map_err(|err| err.extend())?;

                                        if let ConstValue::Null = value {
                                            Ok(FieldValue::NONE)
                                        } else {
                                            Ok(Some(to_field_value(value)))
                                        }
                                    }
                                    .instrument(span)
                                    .inspect_err(|err| tracing::error!(?err)),
                                )
                            }
                        }

                        // --------------------------------------------------
                        //                HOT CODE ENDS HERE
                        // --------------------------------------------------
                        // endregion: hot_code
                    },
                );
                if let Some(description) = &field.description {
                    dyn_schema_field = dyn_schema_field.description(description);
                }
                for arg in field.args.iter() {
                    dyn_schema_field = dyn_schema_field.argument(set_default_value(
                        dynamic::InputValue::new(arg.name.clone(), TypeRef::from(&arg.of_type)),
                        arg.default_value.clone(),
                    ));
                }
                object = object.field(dyn_schema_field);
            }
            if let Some(description) = &def.description {
                object = object.description(description);
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
                    TypeRef::from(&field.of_type),
                ));
            }

            dynamic::Type::Interface(interface)
        }
        Definition::InputObject(def) => {
            let mut input_object = dynamic::InputObject::new(def.name.clone());
            for field in def.fields.iter() {
                let mut input_field =
                    dynamic::InputValue::new(field.name.clone(), TypeRef::from(&field.of_type));
                if let Some(description) = &field.description {
                    input_field = input_field.description(description);
                }
                let input_field = set_default_value(input_field, field.default_value.clone());
                input_object = input_object.field(input_field);
            }
            if let Some(description) = &def.description {
                input_object = input_object.description(description);
            }

            dynamic::Type::InputObject(input_object)
        }
        Definition::Scalar(def) => {
            let mut scalar = dynamic::Scalar::new(def.name.clone());
            if let Some(description) = &def.description {
                scalar = scalar.description(description);
            }
            let name = def.scalar.clone();
            scalar = scalar.validator(move |v| name.validate(v));
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

        schema = inject_custom_scalars(schema, blueprint);

        for def in blueprint.definitions.iter() {
            schema = schema.register(to_type(def));
        }

        schema
    }
}

fn inject_custom_scalars(mut schema: SchemaBuilder, blueprint: &Blueprint) -> SchemaBuilder {
    fn inject_scalar(schema: SchemaBuilder, type_name: &str) -> SchemaBuilder {
        if let Some(scalar) = Scalar::find(type_name) {
            let scalar = scalar.clone();
            schema.register(dynamic::Type::Scalar(
                dynamic::Scalar::new(scalar.name()).validator(move |val| scalar.validate(val)),
            ))
        } else {
            schema
        }
    }

    for ty in blueprint.definitions.clone() {
        match ty {
            Definition::Interface(interface_type_definition) => {
                for field in interface_type_definition.fields {
                    let type_name = field.of_type.name();
                    schema = inject_scalar(schema, type_name);
                }
            }
            Definition::Object(object_type_definition) => {
                for field in object_type_definition.fields {
                    let type_name = field.of_type.name();
                    schema = inject_scalar(schema, type_name);
                }
            }
            Definition::InputObject(input_object_type_definition) => {
                for field in input_object_type_definition.fields {
                    let type_name = field.of_type.name();
                    schema = inject_scalar(schema, type_name);
                }
            }
            _ => (),
        }
    }
    schema
}
