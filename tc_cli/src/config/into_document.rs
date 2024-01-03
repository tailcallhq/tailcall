use async_graphql::parser::types::*;
use async_graphql::{Pos, Positioned};
use async_graphql_value::{ConstValue, Name};
use tc_core::directive::DirectiveCodec;

use super::Config;
use crate::blueprint::TypeLike;

fn pos<A>(a: A) -> Positioned<A> {
  Positioned::new(a, Pos::default())
}
fn config_document(config: &Config) -> ServiceDocument {
  let mut definitions = Vec::new();
  let schema_definition = SchemaDefinition {
    extend: false,
    directives: vec![pos(config.server.to_directive()), pos(config.upstream.to_directive())],
    query: config.schema.query.clone().map(|name| pos(Name::new(name))),
    mutation: config.schema.mutation.clone().map(|name| pos(Name::new(name))),
    subscription: config.schema.subscription.clone().map(|name| pos(Name::new(name))),
  };
  definitions.push(TypeSystemDefinition::Schema(pos(schema_definition)));
  for (type_name, type_def) in config.types.iter() {
    let kind = if type_def.interface {
      TypeKind::Interface(InterfaceType {
        implements: type_def
          .implements
          .iter()
          .map(|name| pos(Name::new(name.clone())))
          .collect(),
        fields: type_def
          .fields
          .clone()
          .iter()
          .map(|(name, field)| {
            let directives = get_directives(field);
            let base_type = if field.list {
              BaseType::List(Box::new(Type {
                nullable: !field.list_type_required,
                base: BaseType::Named(Name::new(field.type_of.clone())),
              }))
            } else {
              BaseType::Named(Name::new(field.type_of.clone()))
            };
            pos(FieldDefinition {
              description: field.doc.clone().map(pos),
              name: pos(Name::new(name.clone())),
              arguments: vec![],
              ty: pos(Type { nullable: !field.required, base: base_type }),

              directives,
            })
          })
          .collect::<Vec<Positioned<FieldDefinition>>>(),
      })
    } else if let Some(variants) = &type_def.variants {
      TypeKind::Enum(EnumType {
        values: variants
          .iter()
          .map(|value| {
            pos(EnumValueDefinition { description: None, value: pos(Name::new(value.clone())), directives: Vec::new() })
          })
          .collect(),
      })
    } else if config.input_types().contains(type_name) {
      TypeKind::InputObject(InputObjectType {
        fields: type_def
          .fields
          .clone()
          .iter()
          .map(|(name, field)| {
            let directives = get_directives(field);
            let base_type = if field.list {
              async_graphql::parser::types::BaseType::List(Box::new(Type {
                nullable: !field.list_type_required,
                base: async_graphql::parser::types::BaseType::Named(Name::new(field.type_of.clone())),
              }))
            } else {
              async_graphql::parser::types::BaseType::Named(Name::new(field.type_of.clone()))
            };

            pos(async_graphql::parser::types::InputValueDefinition {
              description: field.doc.clone().map(pos),
              name: pos(Name::new(name.clone())),
              ty: pos(Type { nullable: !field.required, base: base_type }),

              default_value: None,
              directives,
            })
          })
          .collect::<Vec<Positioned<InputValueDefinition>>>(),
      })
    } else if type_def.fields.is_empty() {
      TypeKind::Scalar
    } else {
      TypeKind::Object(ObjectType {
        implements: type_def
          .implements
          .iter()
          .map(|name| pos(Name::new(name.clone())))
          .collect(),
        fields: type_def
          .fields
          .clone()
          .iter()
          .map(|(name, field)| {
            let directives = get_directives(field);
            let base_type = if field.list {
              async_graphql::parser::types::BaseType::List(Box::new(Type {
                nullable: !field.list_type_required,
                base: async_graphql::parser::types::BaseType::Named(Name::new(field.type_of.clone())),
              }))
            } else {
              async_graphql::parser::types::BaseType::Named(Name::new(field.type_of.clone()))
            };

            let args_map = field.args.clone();
            let args = args_map
              .iter()
              .map(|(name, arg)| {
                let base_type = if arg.list {
                  async_graphql::parser::types::BaseType::List(Box::new(Type {
                    nullable: !arg.list_type_required(),
                    base: async_graphql::parser::types::BaseType::Named(Name::new(arg.type_of.clone())),
                  }))
                } else {
                  async_graphql::parser::types::BaseType::Named(Name::new(arg.type_of.clone()))
                };
                pos(async_graphql::parser::types::InputValueDefinition {
                  description: arg.doc.clone().map(pos),
                  name: pos(Name::new(name.clone())),
                  ty: pos(Type { nullable: !arg.required, base: base_type }),

                  default_value: arg
                    .default_value
                    .clone()
                    .map(|v| pos(ConstValue::String(v.to_string()))),
                  directives: Vec::new(),
                })
              })
              .collect::<Vec<Positioned<InputValueDefinition>>>();

            pos(async_graphql::parser::types::FieldDefinition {
              description: field.doc.clone().map(pos),
              name: pos(Name::new(name.clone())),
              arguments: args,
              ty: pos(Type { nullable: !field.required, base: base_type }),

              directives,
            })
          })
          .collect::<Vec<Positioned<FieldDefinition>>>(),
      })
    };
    definitions.push(TypeSystemDefinition::Type(pos(TypeDefinition {
      extend: false,
      description: None,
      name: pos(Name::new(type_name.clone())),
      directives: type_def
        .added_fields
        .iter()
        .map(|added_field| pos(added_field.to_directive()))
        .collect::<Vec<_>>(),
      kind,
    })));
  }
  for (name, union) in config.unions.iter() {
    definitions.push(TypeSystemDefinition::Type(pos(TypeDefinition {
      extend: false,
      description: None,
      name: pos(Name::new(name)),
      directives: Vec::new(),
      kind: TypeKind::Union(UnionType {
        members: union.types.iter().map(|name| pos(Name::new(name.clone()))).collect(),
      }),
    })));
  }

  ServiceDocument { definitions }
}

fn get_directives(field: &crate::config::Field) -> Vec<Positioned<ConstDirective>> {
  let directives = vec![
    field.http.as_ref().map(|d| pos(d.to_directive())),
    field.unsafe_operation.as_ref().map(|d| pos(d.to_directive())),
    field.const_field.as_ref().map(|d| pos(d.to_directive())),
    field.modify.as_ref().map(|d| pos(d.to_directive())),
    field.graphql.as_ref().map(|d| pos(d.to_directive())),
    field.grpc.as_ref().map(|d| pos(d.to_directive())),
  ];

  directives.into_iter().flatten().collect()
}

impl From<Config> for ServiceDocument {
  fn from(value: Config) -> Self {
    config_document(&value)
  }
}
