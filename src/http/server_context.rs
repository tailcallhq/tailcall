use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dynamic;
use async_graphql_value::ConstValue;

use super::DataLoaderRequest;
use crate::blueprint::Type::ListType;
use crate::blueprint::{Blueprint, Definition};
use crate::chrono_cache::ChronoCache;
use crate::data_loader::DataLoader;
use crate::graphql::GraphqlDataLoader;
use crate::grpc;
use crate::grpc::data_loader::GrpcDataLoader;
use crate::http::HttpDataLoader;
use crate::io::http::HttpIO;
use crate::lambda::{DataLoaderId, Expression, Unsafe};

pub struct ServerContext {
  pub schema: dynamic::Schema,
  pub universal_http_client: Arc<dyn HttpIO>,
  pub http2_only_client: Arc<dyn HttpIO>,
  pub blueprint: Blueprint,
  pub http_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, HttpDataLoader>>>,
  pub gql_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, GraphqlDataLoader>>>,
  pub cache: ChronoCache<u64, ConstValue>,
  pub grpc_data_loaders: Arc<Vec<DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>>>,
  pub env_vars: Arc<HashMap<String, String>>,
}

impl ServerContext {
  pub fn new(
    blueprint: Blueprint,
    universal_http_client: Arc<impl HttpIO + 'static>,
    http2_only_client: Arc<impl HttpIO + 'static>,
  ) -> Self {
    Self::with_http_clients(blueprint, universal_http_client, http2_only_client)
  }

  pub fn with_http_clients(
    mut blueprint: Blueprint,
    universal_http_client: Arc<dyn HttpIO>,
    http2_only_client: Arc<dyn HttpIO>,
  ) -> Self {
    let mut http_data_loaders = vec![];
    let mut gql_data_loaders = vec![];
    let mut grpc_data_loaders = vec![];

    for def in blueprint.definitions.iter_mut() {
      if let Definition::ObjectTypeDefinition(def) = def {
        for field in &mut def.fields {
          if let Some(Expression::Unsafe(expr_unsafe)) = &mut field.resolver {
            match expr_unsafe {
              Unsafe::Http { req_template, group_by, .. } => {
                let data_loader = HttpDataLoader::new(
                  universal_http_client.clone(),
                  group_by.clone(),
                  matches!(&field.of_type, ListType { .. }),
                )
                .to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());

                field.resolver = Some(Expression::Unsafe(Unsafe::Http {
                  req_template: req_template.clone(),
                  group_by: group_by.clone(),
                  dl_id: Some(DataLoaderId(http_data_loaders.len())),
                }));

                http_data_loaders.push(data_loader);
              }

              Unsafe::GraphQLEndpoint { req_template, field_name, batch, .. } => {
                let graphql_data_loader = GraphqlDataLoader::new(universal_http_client.clone(), *batch)
                  .to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());

                field.resolver = Some(Expression::Unsafe(Unsafe::GraphQLEndpoint {
                  req_template: req_template.clone(),
                  field_name: field_name.clone(),
                  batch: *batch,
                  dl_id: Some(DataLoaderId(gql_data_loaders.len())),
                }));

                gql_data_loaders.push(graphql_data_loader);
              }

              Unsafe::Grpc { req_template, group_by, .. } => {
                let data_loader = GrpcDataLoader {
                  client: http2_only_client.clone(),
                  operation: req_template.operation.clone(),
                  group_by: group_by.clone(),
                };
                let data_loader = data_loader.to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());

                field.resolver = Some(Expression::Unsafe(Unsafe::Grpc {
                  req_template: req_template.clone(),
                  group_by: group_by.clone(),
                  dl_id: Some(DataLoaderId(grpc_data_loaders.len())),
                }));

                grpc_data_loaders.push(data_loader);
              }
              _ => {}
            }
          }
        }
      }
    }

    let schema = blueprint.to_schema();

    #[cfg(feature = "default")]
    let env_vars = Arc::new(std::env::vars().collect());
    #[cfg(not(feature = "default"))]
    let env_vars = Arc::new(HashMap::new());

    ServerContext {
      schema,
      universal_http_client,
      http2_only_client,
      blueprint,
      http_data_loaders: Arc::new(http_data_loaders),
      gql_data_loaders: Arc::new(gql_data_loaders),
      cache: ChronoCache::new(),
      grpc_data_loaders: Arc::new(grpc_data_loaders),
      env_vars,
    }
  }
}
