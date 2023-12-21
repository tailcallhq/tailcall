use std::borrow::Cow;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use async_graphql::dynamic::{
  FieldFuture, FieldValue, ResolverContext, SchemaBuilder, {self},
};
use async_graphql_value::ConstValue;

use super::hash_const_value;
use crate::blueprint::{Blueprint, Cache, Definition, Type};
use crate::http::RequestContext;
use crate::json::JsonLike;
use crate::lambda::EvaluationContext;

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

fn get_cache_key<'a, H: Hasher + Clone>(ctx: &'a EvaluationContext<'a, ResolverContext<'a>>, mut hasher: H) -> Option<u64> {
  // Hash on parent value
  if let Some(const_value) = ctx
    .graphql_ctx
    .parent_value
    .as_value()
    // TODO: handle _id, id, or any field that has @key on it.
    .map(|data| data.get_key("id"))
  {
    // Hash on parent's id only?
    hash_const_value(const_value?, &mut hasher)
  }

  let key = ctx
    .graphql_ctx
    .args
    .iter()
    .map(|(key, value)| {
      let mut hasher = hasher.clone();
      key.hash(&mut hasher);
      hash_const_value(value.as_value(), &mut hasher);
      hasher.finish()
    })
    .fold(hasher.finish(), |acc, val| acc ^ val);

  Some(key)
}

fn to_type(def: &Definition) -> dynamic::Type {
  match def {
    Definition::ObjectTypeDefinition(def) => {
      let mut object = dynamic::Object::new(def.name.clone());
      for field in def.fields.iter() {
        let field = field.clone();
        let type_ref = to_type_ref(&field.of_type);
        let field_name = &field.name.clone();
        let cache = field.cache.clone();
        let mut dyn_schema_field = dynamic::Field::new(field_name, type_ref, move |ctx| {
          let req_ctx = ctx.ctx.data::<Arc<RequestContext>>().unwrap();
          let field_name = &field.name;
          match &field.resolver {
            None => {
              let ctx = EvaluationContext::new(req_ctx, &ctx);
              FieldFuture::from_value(ctx.path_value(&[field_name]).map(|a| a.to_owned()))
            }
            Some(expr) => {
              let expr = expr.to_owned();
              let cache = cache.clone();
              FieldFuture::new(async move {
                let ctx = EvaluationContext::new(req_ctx, &ctx);

                let ttl_and_key = cache.and_then(|Cache { max_age: ttl, hasher }| Some((ttl, get_cache_key(&ctx, hasher)?)));
                let const_value = match ttl_and_key {
                  Some((ttl, key)) => {
                    if let Some(const_value) = ctx.req_ctx.cache_get(&key) {
                      // Return value from cache
                      log::info!("Reading from cache. key = {key}");
                      const_value
                    } else {
                      let const_value = expr.eval(&ctx).await?;
                      log::info!("Writing to cache. key = {key}");
                      // Write value to cache
                      ctx.req_ctx.cache_insert(key, const_value.clone(), ttl);
                      const_value
                    }
                  }
                  _ => expr.eval(&ctx).await?,
                };

                let p = match const_value {
                  ConstValue::List(a) => FieldValue::list(a),
                  a => FieldValue::from(a),
                };
                Ok(Some(p))
              })
            }
          }
        });
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
    schema = schema.register(to_type(def));
  }

  schema
}

impl From<&Blueprint> for SchemaBuilder {
  fn from(blueprint: &Blueprint) -> Self {
    create(blueprint)
  }
}
