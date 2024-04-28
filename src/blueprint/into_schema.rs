use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dynamic::{self, FieldFuture, FieldValue, SchemaBuilder};
use async_graphql::SelectionField;
use async_graphql_value::ConstValue;
use futures_util::TryFutureExt;
use tracing::Instrument;

use crate::blueprint::{Blueprint, Definition, FieldDefinition, Type};
use crate::http::RequestContext;
use crate::lambda::{Concurrent, Eval, EvaluationContext, ResolverContext};
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

fn to_type(
    def: &Definition,
    type_map: HashMap<String, HashMap<String, FieldDefinition>>,
) -> dynamic::Type {
    let type_map = Arc::new(type_map);
    match def {
        Definition::Object(def) => {
            let mut object = dynamic::Object::new(def.name.clone());
            for field in def.fields.iter() {
                let field = field.clone();
                let type_ref = to_type_ref(&field.of_type);
                let field_name = &field.name.clone();
                let type_map = type_map.clone();
                let mut dyn_schema_field = dynamic::Field::new(
                    field_name,
                    type_ref.clone(),
                    move |ctx| {
                        let req_ctx = ctx.ctx.data::<Arc<RequestContext>>().unwrap();
                        let field_name = &field.name;

                        match &field.resolver {
                            None => {
                                let ctx: ResolverContext = ctx.into();
                                let ctx = EvaluationContext::new(req_ctx, &ctx);
                                FieldFuture::from_value(
                                    ctx.path_value(&[field_name])
                                        .map(|a| a.into_owned().to_owned()),
                                )
                            }
                            Some(expr) => {
                                let span = tracing::info_span!(
                                    "field_resolver",
                                    otel.name = ctx.path_node.map(|p| p.to_string()).unwrap_or(field_name.clone()), graphql.returnType = %type_ref
                                );
                                let expr = expr.to_owned();
                                let type_map = type_map.clone();
                                let of_type = field
                                    .of_type
                                    .is_list()
                                    .then_some(field.of_type.name())
                                    .map(String::from);
                                FieldFuture::new(
                                    async move {
                                        let mut enable_batching = false;
                                        if let Some(typ) = of_type {
                                            if check_field_has_io_resolver(
                                                &typ,
                                                ctx.field().selection_set(),
                                                type_map.as_ref(),
                                            ) {
                                                enable_batching = true;
                                            }
                                        }

                                        let mut ctx: ResolverContext = ctx.into();
                                        if enable_batching {
                                            ctx.enable_batching();
                                        }
                                        let ctx = EvaluationContext::new(req_ctx, &ctx);

                                        let const_value =
                                            expr.eval(ctx, &Concurrent::Sequential).await?;
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

fn check_field_has_io_resolver<'a>(
    type_name: &str,
    selection_fields: impl Iterator<Item = SelectionField<'a>>,
    type_map: &HashMap<String, HashMap<String, FieldDefinition>>,
) -> bool {
    for field in selection_fields {
        if let Some(typ) = type_map.get(type_name) {
            if let Some(fld) = typ.get(field.name()) {
                if fld.resolver.is_some()
                    || check_field_has_io_resolver(
                        fld.of_type.name(),
                        field.selection_set(),
                        type_map,
                    )
                {
                    return true;
                }
            }
        }
    }

    false
}

fn create_type_map(defs: &[Definition]) -> HashMap<String, HashMap<String, FieldDefinition>> {
    defs.iter()
        .filter_map(|def| match def {
            Definition::Object(obj) => {
                let fld_map = obj
                    .fields
                    .iter()
                    .map(|fld| (fld.name.clone(), fld.clone()))
                    .collect();
                Some((obj.name.clone(), fld_map))
            }
            _ => None,
        })
        .collect()
}

impl From<&Blueprint> for SchemaBuilder {
    fn from(blueprint: &Blueprint) -> Self {
        let query = blueprint.query();
        let mutation = blueprint.mutation();
        let mut schema = dynamic::Schema::build(query.as_str(), mutation.as_deref(), None);

        let type_map = create_type_map(&blueprint.definitions);

        for (k, v) in CUSTOM_SCALARS.iter() {
            schema = schema.register(dynamic::Type::Scalar(
                dynamic::Scalar::new(k.clone()).validator(v.validate()),
            ));
        }

        for def in blueprint.definitions.iter() {
            schema = schema.register(to_type(def, type_map.clone()));
        }

        schema
    }
}
