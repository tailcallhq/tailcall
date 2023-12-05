use std::sync::Arc;

use async_graphql::dataloader::DataLoader;
use async_graphql::dynamic;

use super::HttpClient;
use crate::blueprint::Type::ListType;
use crate::blueprint::{Blueprint, Definition};
use crate::http::{GraphqlDataLoader, HttpDataLoader};
use crate::lambda::{Expression, Unsafe};

pub struct ServerContext {
  pub schema: dynamic::Schema,
  pub http_client: Arc<dyn HttpClient>,
  pub blueprint: Blueprint,
  pub http_data_loaders: Arc<Vec<DataLoader<HttpDataLoader>>>,
  pub gql_data_loaders: Arc<Vec<DataLoader<GraphqlDataLoader>>>,
}

fn assign_data_loaders<'a>(
  blueprint: &'a mut Blueprint,
  http_client: Arc<dyn HttpClient>,
  http_data_loaders: &mut Vec<DataLoader<HttpDataLoader>>,
  gql_data_loaders: &mut Vec<DataLoader<GraphqlDataLoader>>,
) -> &'a Blueprint {
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
                Some(http_data_loaders.len()),
              )));

              http_data_loaders.push(data_loader);
            }

            Unsafe::GraphQLEndpoint { req_template, field_name, batch, .. } => {
              let graphql_data_loader = GraphqlDataLoader::new(http_client.clone(), *batch)
                .to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());

              field.resolver = Some(Expression::Unsafe(Unsafe::GraphQLEndpoint {
                req_template: req_template.clone(),
                field_name: field_name.clone(),
                batch: *batch,
                data_loader_index: Some(gql_data_loaders.len()),
              }));

              gql_data_loaders.push(graphql_data_loader);
            }
            _ => {}
          }
        }
      }
    }
  }
  blueprint
}

impl ServerContext {
  pub fn new(mut blueprint: Blueprint, http_client: Arc<dyn HttpClient>) -> Self {
    let mut http_data_loaders = vec![];
    let mut gql_data_loaders = vec![];
    let schema = assign_data_loaders(
      &mut blueprint,
      http_client.clone(),
      &mut http_data_loaders,
      &mut gql_data_loaders,
    )
    .to_schema();
    let http_data_loaders = Arc::new(http_data_loaders);
    let gql_data_loaders = Arc::new(gql_data_loaders);
    ServerContext { schema, http_client, blueprint, http_data_loaders, gql_data_loaders }
  }
}
