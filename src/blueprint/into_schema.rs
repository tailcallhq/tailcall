use std::borrow::Cow;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use async_graphql::dynamic::{
  FieldFuture, FieldValue, ResolverContext, SchemaBuilder, {self},
};
use async_graphql_value::ConstValue;

use crate::blueprint::{Blueprint, Definition, Type};
use crate::http::RequestContext;
use crate::json::JsonLike;
use crate::lambda::{EvaluationContext, Expression};

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

fn get_type_from_value(value: &ConstValue, field_name: String) -> Result<String> {
  let path = vec!["__typename".to_string()];
  let value = value
    .get_path(&path)
    .ok_or(anyhow!("Could not find __typename in result for field {}", field_name))?;
  let type_ = match value {
    ConstValue::String(s) => s.clone(),
    _ => "".to_string(),
  };
  Ok(type_)
}

fn create_list_value_with_type(values: Vec<ConstValue>, field_name: String) -> Result<FieldValue<'static>> {
  let values = values
    .iter()
    .map(|item| create_field_value_with_type(item.clone(), field_name.clone()))
    .collect::<Vec<_>>();
  let list_values: Result<Vec<FieldValue<'_>>> = values.into_iter().collect();
  Ok(FieldValue::list(list_values?))
}

fn create_field_value_with_type(value: ConstValue, field_name: String) -> Result<FieldValue<'static>> {
  let typename = get_type_from_value(&value, field_name)?;
  let field_value = FieldValue::from(value);
  Ok(FieldValue::with_type(field_value, typename))
}

async fn get_evaled_value_from_ctx<'a>(
  req_ctx: &'a Arc<RequestContext>,
  ctx: &'a ResolverContext<'_>,
  expr: Expression,
) -> Result<async_graphql::Value> {
  let ctx = EvaluationContext::new(req_ctx, ctx);
  expr.eval(&ctx).await
}

fn get_field_value_from_ctx<'a>(
  req_ctx: &'a Arc<RequestContext>,
  ctx: &'a ResolverContext<'_>,
  field_name: String,
) -> Option<FieldValue<'static>> {
  let ctx = EvaluationContext::new(req_ctx, ctx);
  ctx.path_value(&[field_name]).map(|a| FieldValue::from(a.to_owned()))
}

fn to_type(def: &Definition, definitions: &Vec<Definition>) -> dynamic::Type {
  match def {
    Definition::ObjectTypeDefinition(def) => {
      let mut object = dynamic::Object::new(def.name.clone());
      for field in def.fields.iter() {
        let field = field.clone();
        let type_ref = to_type_ref(&field.of_type);
        let field_name = &field.name.clone();
        let is_interface_type = is_interface_type(field.of_type.name(), definitions);

        let mut dyn_schema_field = if !is_interface_type {
          dynamic::Field::new(field_name, type_ref, move |ctx| {
            let req_ctx = ctx.ctx.data::<Arc<RequestContext>>().unwrap();
            let field_name = field.name.clone();
            let resolver = field.resolver.clone();
            FieldFuture::new(async move {
              match resolver {
                None => Ok(get_field_value_from_ctx(req_ctx, &ctx, field_name)),
                Some(expr) => {
                  let const_value = get_evaled_value_from_ctx(req_ctx, &ctx, expr).await?;
                  let p = match const_value {
                    ConstValue::List(a) => FieldValue::list(a),
                    a => FieldValue::from(a),
                  };
                  Ok(Some(p))
                }
              }
            })
          })
        } else {
          dynamic::Field::new(field_name, type_ref, move |ctx| {
            let req_ctx = ctx.ctx.data::<Arc<RequestContext>>().unwrap();
            let field_name = field.name.clone();
            let resolver = field.resolver.clone();
            FieldFuture::new(async move {
              match resolver {
                None => Ok(get_field_value_from_ctx(req_ctx, &ctx, field_name)),
                Some(expr) => {
                  let const_value = get_evaled_value_from_ctx(req_ctx, &ctx, expr).await?;
                  let p = match const_value {
                    ConstValue::List(a) => create_list_value_with_type(a, field_name)?,
                    a => create_field_value_with_type(a, field_name)?,
                  };
                  Ok(Some(p))
                }
              }
            })
          })
        };
        if let Some(description) = &field.description {
          dyn_schema_field = dyn_schema_field.description(description);
        }
        for arg in field.args.iter() {
          dyn_schema_field =
            dyn_schema_field.argument(dynamic::InputValue::new(arg.name.clone(), to_type_ref(&arg.of_type)));
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

fn create(blueprint: &Blueprint) -> SchemaBuilder {
  let query = blueprint.query();
  let mutation = blueprint.mutation();
  let mut schema = dynamic::Schema::build(query.as_str(), mutation.as_deref(), None);

  for def in blueprint.definitions.iter() {
    schema = schema.register(to_type(def, &blueprint.definitions));
  }

  schema
}

fn is_interface_type(name: &str, definitions: &Vec<Definition>) -> bool {
  for def in definitions {
    if def.name() == name {
      if let Definition::InterfaceTypeDefinition(_) = def {
        return true;
      }
    }
  }
  false
}

impl From<&Blueprint> for SchemaBuilder {
  fn from(blueprint: &Blueprint) -> Self {
    create(blueprint)
  }
}
