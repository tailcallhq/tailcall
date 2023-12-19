use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use async_graphql::dynamic::{
  FieldFuture, FieldValue, SchemaBuilder, {self},
};
use async_graphql_value::ConstValue;

use crate::blueprint::{Blueprint, Definition, Type};
use crate::graphql::ResCache;
use crate::http::RequestContext;
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

fn hash_const_value<H: Hasher>(const_value: &ConstValue, state: &mut H) {
  match const_value {
    ConstValue::Null => {}
    ConstValue::Boolean(val) => val.hash(state),
    ConstValue::Enum(name) => name.hash(state),
    ConstValue::Number(num) => num.hash(state),
    ConstValue::Binary(bytes) => bytes.hash(state),
    ConstValue::String(string) => string.hash(state),
    ConstValue::List(list) => list.iter().for_each(|val| hash_const_value(val, state)),
    ConstValue::Object(object) => {
      let mut tmp_list: Vec<_> = object.iter().collect();
      tmp_list.sort_by(|(key1, _), (key2, _)| key1.cmp(key2));
      tmp_list.iter().for_each(|(key, value)| {
        key.hash(state);
        hash_const_value(value, state);
      })
    }
  }
}

fn to_type(def: &Definition) -> dynamic::Type {
  match def {
    Definition::ObjectTypeDefinition(def) => {
      let mut object = dynamic::Object::new(def.name.clone());
      for field in def.fields.iter() {
        let field = field.clone();
        let type_ref = to_type_ref(&field.of_type);
        let field_name = &field.name.clone();
        let def_name = def.name.clone();
        let cache = field.cache.clone();
        // FIXME: Create a global res_cache once and use it for all fields
        let res_cache = ResCache::new(cache.unwrap());
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
              let def_name = def_name.clone();
              let field_name = field_name.clone();
              let res_cache = res_cache.clone();
              FieldFuture::new(async move {
                let ctx = EvaluationContext::new(req_ctx, &ctx);
                let mut hasher = DefaultHasher::new();
                let state = &mut hasher;
                def_name.hash(state);
                field_name.hash(state);

                if let Some(const_value) = ctx.graphql_ctx.parent_value.as_value() {
                  hash_const_value(const_value, state)
                }

                let mut args_list: Vec<_> = ctx.graphql_ctx.args.iter().collect();
                args_list.sort_by(|(key1, _), (key2, _)| key1.cmp(key2));
                args_list.iter().for_each(|(key, value)| {
                  key.hash(state);
                  hash_const_value(value.as_value(), state);
                });

                let key = hasher.finish();
                let const_value = res_cache.fetch(&ctx, &expr, key).await?;

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
