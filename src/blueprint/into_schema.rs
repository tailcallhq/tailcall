use std::borrow::Cow;

use std::sync::Arc;

use async_graphql::dataloader::{DataLoader, HashMapCache};
use async_graphql::dynamic::{self, FieldFuture, SchemaBuilder};

use crate::blueprint::{Blueprint, Type};
use crate::blueprint::{Definition, SchemaDefinition};

use crate::evaluation_context::EvaluationContext;
use crate::http::HttpDataLoader;

fn to_type_ref(type_of: &Type) -> dynamic::TypeRef {
    match type_of {
        Type::NamedType { name, non_null } => {
            if *non_null {
                dynamic::TypeRef::NonNull(Box::from(dynamic::TypeRef::Named(Cow::Owned(name.clone()))))
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

fn to_type(def: &Definition, schema0: SchemaDefinition) -> dynamic::Type {
    match def {
        Definition::ObjectTypeDefinition(def) => {
            let mut object = dynamic::Object::new(def.name.clone());
            for field in def.fields.iter() {
                let cloned_field = field.clone();
                let schema = schema0.clone();
                let mut dyn_schema_field =
                    dynamic::Field::new(field.name.clone(), to_type_ref(&field.of_type), move |ctx| {
                        let loader = ctx.ctx.data::<Arc<DataLoader<HttpDataLoader, HashMapCache>>>().unwrap();
                        let field_name = cloned_field.name.clone();
                        let resolver = cloned_field.clone().resolver;
                        let schema = schema.clone();
                        FieldFuture::new(async move {
                            match resolver {
                                None => Ok(ctx.parent_value.as_value().and_then(|value| match value {
                                    async_graphql::Value::Object(map) => {
                                        map.get(&async_graphql::Name::new(field_name)).cloned()
                                    }
                                    _ => None,
                                })),
                                Some(expr) => {
                                    let mut ctx = EvaluationContext::new(loader).context(&ctx);
                                    if let Some(vars) = schema.clone().directives.first().cloned().map(|x| x.arguments)
                                    {
                                        ctx = ctx.env(vars);
                                    }
                                    let async_value = expr.eval(&ctx).await?;

                                    Ok(Some(async_value))
                                }
                            }
                        })
                    });
                for arg in field.args.iter() {
                    dyn_schema_field = dyn_schema_field
                        .argument(dynamic::InputValue::new(arg.name.clone(), to_type_ref(&arg.of_type)));
                }
                object = object.field(dyn_schema_field);
            }
            for interface in def.implements.iter() {
                object = object.implement(interface.clone());
            }

            dynamic::Type::Object(object)
        }
        Definition::InterfaceTypeDefinition(def) => {
            let mut interface = dynamic::Interface::new(def.name.clone());
            for field in def.fields.iter() {
                interface = interface.field(dynamic::InterfaceField::new(
                    field.name.clone(),
                    to_type_ref(&field.of_type),
                ));
            }

            dynamic::Type::Interface(interface)
        }
        Definition::InputObjectTypeDefinition(def) => {
            let mut input_object = dynamic::InputObject::new(def.name.clone());
            for field in def.fields.iter() {
                input_object = input_object.field(dynamic::InputValue::new(
                    field.name.clone(),
                    to_type_ref(&field.of_type),
                ));
            }

            dynamic::Type::InputObject(input_object)
        }
        Definition::ScalarTypeDefinition(def) => {
            let mut scalar = dynamic::Scalar::new(def.name.clone());
            if let Some(description) = &def.description {
                scalar = scalar.description(description);
            }
            dynamic::Type::Scalar(scalar)
        }
        Definition::EnumTypeDefinition(def) => {
            let mut enum_type = dynamic::Enum::new(def.name.clone());
            for value in def.enum_values.iter() {
                enum_type = enum_type.item(dynamic::EnumItem::new(value.name.clone()));
            }
            dynamic::Type::Enum(enum_type)
        }
        Definition::UnionTypeDefinition(def) => {
            let mut union = dynamic::Union::new(def.name.clone());
            for type_ in def.types.iter() {
                union = union.possible_type(type_.clone());
            }
            dynamic::Type::Union(union)
        }
    }
}

fn create(blueprint: Blueprint) -> SchemaBuilder {
    let query = blueprint.query();
    let mutation = blueprint.mutation();
    let mut schema = dynamic::Schema::build(query.as_str(), mutation.as_deref(), None);

    for def in blueprint.definitions.iter() {
        schema = schema.register(to_type(def, blueprint.schema.clone()));
    }

    schema
}

impl From<Blueprint> for SchemaBuilder {
    fn from(blueprint: Blueprint) -> Self {
        create(blueprint)
    }
}
