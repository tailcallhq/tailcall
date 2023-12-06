use std::sync::Arc;

use async_graphql::dataloader::DataLoader;
use async_graphql::dynamic;

use super::HttpClient;
use crate::blueprint::Type::ListType;
use crate::blueprint::{Blueprint, Definition};
use crate::http::{GraphqlDataLoader, HttpDataLoader};
use crate::lambda::{DataLoaderId, Expression, Unsafe};

pub struct ServerContext {
  pub schema: dynamic::Schema,
  pub http_client: Arc<dyn HttpClient>,
  pub blueprint: Blueprint,
  pub http_data_loaders: Arc<Vec<DataLoader<HttpDataLoader>>>,
  pub gql_data_loaders: Arc<Vec<DataLoader<GraphqlDataLoader>>>,
}

struct PartialServerContext {
  http_client: Arc<dyn HttpClient>,
  blueprint: Blueprint,
  http_data_loaders: Vec<DataLoader<HttpDataLoader>>,
  gql_data_loaders: Vec<DataLoader<GraphqlDataLoader>>,
}

impl PartialServerContext {
  fn create_server_ctx(mut self) -> ServerContext {
    for def in self.blueprint.definitions.iter_mut() {
      if let Definition::ObjectTypeDefinition(def) = def {
        for field in &mut def.fields {
          if let Some(Expression::Unsafe(expr_unsafe)) = &mut field.resolver {
            match expr_unsafe {
              Unsafe::Http { req_template, group_by, .. } => {
                let data_loader = HttpDataLoader::new(
                  self.http_client.clone(),
                  group_by.clone(),
                  matches!(&field.of_type, ListType { .. }),
                )
                .to_data_loader(self.blueprint.upstream.batch.clone().unwrap_or_default());

                field.resolver = Some(Expression::Unsafe(Unsafe::Http {
                  req_template: req_template.clone(),
                  group_by: group_by.clone(),
                  dl_id: Some(DataLoaderId(self.http_data_loaders.len())),
                }));

                self.http_data_loaders.push(data_loader);
              }

              Unsafe::GraphQLEndpoint { req_template, field_name, batch, .. } => {
                let graphql_data_loader = GraphqlDataLoader::new(self.http_client.clone(), *batch)
                  .to_data_loader(self.blueprint.upstream.batch.clone().unwrap_or_default());

                field.resolver = Some(Expression::Unsafe(Unsafe::GraphQLEndpoint {
                  req_template: req_template.clone(),
                  field_name: field_name.clone(),
                  batch: *batch,
                  dl_id: Some(DataLoaderId(self.gql_data_loaders.len())),
                }));

                self.gql_data_loaders.push(graphql_data_loader);
              }
              _ => {}
            }
          }
        }
      }
    }

    let schema = self.blueprint.to_schema();

    ServerContext {
      schema,
      http_client: self.http_client,
      blueprint: self.blueprint,
      http_data_loaders: Arc::new(self.http_data_loaders),
      gql_data_loaders: Arc::new(self.gql_data_loaders),
    }
  }
}

impl ServerContext {
  pub fn new(blueprint: Blueprint, http_client: Arc<dyn HttpClient>) -> Self {
    let http_data_loaders = vec![];
    let gql_data_loaders = vec![];

    PartialServerContext { http_client, blueprint, http_data_loaders, gql_data_loaders }.create_server_ctx()
  }
}
