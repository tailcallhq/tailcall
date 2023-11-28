use std::collections::BTreeMap;
use std::sync::Arc;

use async_graphql::dynamic;

use super::HttpClient;
use crate::blueprint::Type::ListType;
use crate::blueprint::{Blueprint, Definition};
use crate::config::JoinType;
use crate::http::{GraphqlDataLoader, HttpDataLoader};
use crate::lambda::{Expression, Unsafe};

pub struct ServerContext {
  pub schema: dynamic::Schema,
  pub http_client: Arc<dyn HttpClient>,
  pub blueprint: Blueprint,
}

fn assign_data_loaders(blueprint: &mut Blueprint, http_client: Arc<dyn HttpClient>) -> &Blueprint {
  let mut type_subgraph_fields: BTreeMap<String, (BTreeMap<String, Vec<(String, String)>>, Vec<JoinType>)> =
    BTreeMap::new();

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
              req_template.type_subgraph_fields = type_subgraph_fields.clone();

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
    if let Some(join_field) = &field.join_field {
      if let Some(subgraph_fields) = all_subgraph_fields.get_mut(&join_field.base_url) {
        subgraph_fields.push((field.name.clone(), field.of_type.name().to_string()));
      } else {
        let mut subgraph_fields = Vec::new();
        subgraph_fields.push((field.name.clone(), field.of_type.name().to_string()));
        all_subgraph_fields.insert(join_field.base_url.clone(), subgraph_fields);
      }
    } else if def.join_types.len() == 1 {
      match def.join_types.get(0) {
        Some(join_type) => {
          if let Some(subgraph_fields) = all_subgraph_fields.get_mut(&join_type.base_url.clone().unwrap_or_default()) {
            subgraph_fields.push((field.name.clone(), field.of_type.name().to_string()));
          } else {
            let mut subgraph_fields = Vec::new();
            subgraph_fields.push((field.name.clone(), field.of_type.name().to_string()));
            all_subgraph_fields.insert(join_type.base_url.clone().unwrap_or_default(), subgraph_fields);
          }
        }
        None => {}
      }
    }
  }
}

impl ServerContext {
  pub fn new(mut blueprint: Blueprint, http_client: Arc<dyn HttpClient>) -> Self {
    let schema = assign_data_loaders(&mut blueprint, http_client.clone()).to_schema();
    ServerContext { schema, http_client, blueprint }
  }
}
