use std::sync::Arc;
use std::collections::BTreeMap;

use async_graphql::dynamic;
use crate::http::DefaultHttpClient;

use super::{DataLoaderRequest, HttpClient};
use crate::blueprint::Type::ListType;
use crate::blueprint::{Blueprint, Definition};
use crate::data_loader::DataLoader;
use crate::graphql::GraphqlDataLoader;
use crate::http::HttpDataLoader;
use crate::lambda::{DataLoaderId, Expression, Unsafe};

pub struct ServerContext {
  pub schema: dynamic::Schema,
  pub blueprint: Blueprint,
  pub http_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, HttpDataLoader>>>,
  pub gql_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, GraphqlDataLoader>>>,
}

impl ServerContext {
  pub fn new(mut blueprint: Blueprint, http_clients: BTreeMap<String, Arc<dyn HttpClient>>) -> Self {
    let mut http_data_loaders = vec![];
    let mut gql_data_loaders = vec![];

    for def in blueprint.definitions.iter_mut() {
      if let Definition::ObjectTypeDefinition(def) = def {
        for field in &mut def.fields {
          if let Some(Expression::Unsafe(expr_unsafe)) = &mut field.resolver {
            match expr_unsafe {
              Unsafe::Http { req_template, group_by, upstream, .. } => {
                let data_loader = HttpDataLoader::new(
                  http_clients.get(&upstream.name.clone().unwrap_or("default".to_string())).unwrap().clone(),
                  group_by.clone(),
                  matches!(&field.of_type, ListType { .. }),
                )
                .to_data_loader(upstream.batch.clone().unwrap_or_default());

                field.resolver = Some(Expression::Unsafe(Unsafe::Http {
                  req_template: req_template.clone(),
                  group_by: group_by.clone(),
                  dl_id: Some(DataLoaderId(http_data_loaders.len())),
                  upstream: upstream.clone()
                }));

                http_data_loaders.push(data_loader);
              }

              Unsafe::GraphQLEndpoint { req_template, field_name, batch, upstream, .. } => {
                let graphql_data_loader = GraphqlDataLoader::new(Arc::new(DefaultHttpClient::new(&upstream)), *batch)
                  .to_data_loader(upstream.batch.clone().unwrap_or_default());

                field.resolver = Some(Expression::Unsafe(Unsafe::GraphQLEndpoint {
                  req_template: req_template.clone(),
                  field_name: field_name.clone(),
                  batch: *batch,
                  dl_id: Some(DataLoaderId(gql_data_loaders.len())),
                  upstream: upstream.clone(),
                }));

                gql_data_loaders.push(graphql_data_loader);
              }
              _ => {}
            }
          }
        }
      }
    }

    let schema = blueprint.to_schema();

    ServerContext {
      schema,
      blueprint,
      http_data_loaders: Arc::new(http_data_loaders),
      gql_data_loaders: Arc::new(gql_data_loaders),
    }
  }
}
