use std::collections::BTreeMap;
use std::sync::Arc;

use async_graphql::dynamic;

use super::HttpClient;
use crate::blueprint::Type::ListType;
use crate::blueprint::{Blueprint, Definition};
use crate::config::JoinType;
use crate::http::{GraphqlDataLoader, HttpDataLoader};
use crate::lambda::{Expression, Unsafe, UrlToFieldNameAndTypePairsMap};

pub struct ServerContext {
  pub schema: dynamic::Schema,
  pub http_client: Arc<dyn HttpClient>,
  pub blueprint: Blueprint,
}

fn assign_data_loaders(blueprint: &mut Blueprint, http_client: Arc<dyn HttpClient>) -> &Blueprint {
  for def in blueprint.definitions.iter_mut() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &mut def.fields {
        if let Some(Expression::Unsafe(expr_unsafe)) = &mut field.resolver {
          match expr_unsafe {
            Unsafe::Http(req_template, group_by, _) => {
              let data_loader = HttpDataLoader::new(
                http_client.clone(),
                group_by.clone(),
                matches!(&field.of_type, ListType { .. }),
              )
              .to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());

              field.resolver = Some(Expression::Unsafe(Unsafe::Http(
                req_template.clone(),
                group_by.clone(),
                Some(Arc::new(data_loader)),
              )));
            }

            Unsafe::GraphQLEndpoint { req_template, field_name, batch, .. } => {
              let graphql_data_loader = GraphqlDataLoader::new(http_client.clone(), *batch)
                .to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());

              field.resolver = Some(Expression::Unsafe(Unsafe::GraphQLEndpoint {
                req_template: req_template.clone(),
                field_name: field_name.clone(),
                batch: *batch,
                data_loader: Some(Arc::new(graphql_data_loader)),
              }));
            }
            _ => {}
          }
        }
      }
    }
  }
  blueprint
}

fn update_fields_for_type(
  def: &mut crate::blueprint::ObjectTypeDefinition,
  all_subgraph_fields: &mut BTreeMap<String, Vec<(String, String)>>,
) {
  for field in &mut def.fields {
    if let Some(Expression::Unsafe(Unsafe::GraphQLEndpoint { req_template, .. })) = &field.resolver {
      if req_template.federate {
        if let Some(subgraph_fields) = all_subgraph_fields.get_mut(&req_template.url) {
          subgraph_fields.push((field.name.clone(), field.of_type.name().to_string()));
        } else {
          let subgraph_fields = vec![(field.name.clone(), field.of_type.name().to_string())];
          all_subgraph_fields.insert(req_template.url.clone(), subgraph_fields);
        }
      } else {
        update_field_from_join_type(&def.join_types, all_subgraph_fields, field);
      }
    } else {
      update_field_from_join_type(&def.join_types, all_subgraph_fields, field);
    }
  }
}

fn update_field_from_join_type(
  join_types: &[JoinType],
  all_subgraph_fields: &mut BTreeMap<String, Vec<(String, String)>>,
  field: &crate::blueprint::FieldDefinition,
) {
  for join_type in join_types.iter() {
    if let Some(subgraph_fields) = all_subgraph_fields.get_mut(&join_type.base_url.clone().unwrap_or_default()) {
      subgraph_fields.push((field.name.clone(), field.of_type.name().to_string()));
    } else {
      let subgraph_fields = vec![(field.name.clone(), field.of_type.name().to_string())];
      all_subgraph_fields.insert(join_type.base_url.clone().unwrap_or_default(), subgraph_fields);
    }
  }
}

fn assign_url_type_fields(blueprint: &mut Blueprint) -> &mut Blueprint {
  let mut type_subgraph_fields: BTreeMap<String, (UrlToFieldNameAndTypePairsMap, Vec<JoinType>)> = BTreeMap::new();

  for def in blueprint.definitions.iter_mut() {
    if let Definition::ObjectTypeDefinition(def) = def {
      if let Some(all_subgraph_fields) = type_subgraph_fields.get_mut(&def.name) {
        update_fields_for_type(def, &mut all_subgraph_fields.0);
      } else {
        let mut all_subgraph_fields = BTreeMap::new();
        update_fields_for_type(def, &mut all_subgraph_fields);
        type_subgraph_fields.insert(def.name.clone(), (all_subgraph_fields, def.join_types.clone()));
      }
    }
  }

  for def in blueprint.definitions.iter_mut() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &mut def.fields {
        if let Some(Expression::Unsafe(Unsafe::GraphQLEndpoint { req_template, .. })) = &mut field.resolver {
          req_template.type_subgraph_fields = type_subgraph_fields.clone();
        }
      }
    }
  }
  blueprint
}

impl ServerContext {
  pub fn new(mut blueprint: Blueprint, http_client: Arc<dyn HttpClient>) -> Self {
    let blueprint = assign_url_type_fields(&mut blueprint);
    let schema = assign_data_loaders(blueprint, http_client.clone()).to_schema();
    ServerContext { schema, http_client, blueprint: blueprint.clone() }
  }
}
