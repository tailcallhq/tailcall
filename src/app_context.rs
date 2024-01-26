#![deny(clippy::wildcard_enum_match_arm)] // to make sure we handle all of the expression variants

use std::sync::Arc;

use async_graphql::dynamic::{self, DynamicRequest};
use async_graphql::Response;

use crate::blueprint::Type::ListType;
use crate::blueprint::{Blueprint, Definition};
use crate::data_loader::DataLoader;
use crate::graphql::GraphqlDataLoader;
use crate::grpc::data_loader::GrpcDataLoader;
use crate::http::{DataLoaderRequest, HttpDataLoader};
use crate::lambda::{DataLoaderId, Expression, IO};
use crate::{grpc, EntityCache, EnvIO, HttpIO};

pub struct AppContext<Http, Env> {
  pub schema: dynamic::Schema,
  pub universal_http_client: Arc<Http>,
  pub http2_only_client: Arc<Http>,
  pub blueprint: Blueprint,
  pub http_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, HttpDataLoader>>>,
  pub gql_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, GraphqlDataLoader>>>,
  pub grpc_data_loaders: Arc<Vec<DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>>>,
  pub cache: Arc<EntityCache>,
  pub env_vars: Arc<Env>,
}

impl<Http: HttpIO, Env: EnvIO> AppContext<Http, Env> {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    mut blueprint: Blueprint,
    h_client: Arc<Http>,
    h2_client: Arc<Http>,
    env: Arc<Env>,
    cache: Arc<EntityCache>,
  ) -> Self {
    let mut http_data_loaders = vec![];
    let mut gql_data_loaders = vec![];
    let mut grpc_data_loaders = vec![];

    for def in blueprint.definitions.iter_mut() {
      if let Definition::ObjectTypeDefinition(def) = def {
        for field in &mut def.fields {
          if let Some(expr) = &mut field.resolver {
            expr.for_each_mut(&mut |expr| {
              if let Expression::IO(io) = expr {
                match io {
                  IO::Http { group_by, dl_id, .. } => {
                    let data_loader = HttpDataLoader::new(
                      h_client.clone(),
                      group_by.clone(),
                      matches!(&field.of_type, ListType { .. }),
                    )
                    .to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());

                    let _ = dl_id.insert(DataLoaderId(http_data_loaders.len()));

                    http_data_loaders.push(data_loader);
                  }

                  IO::GraphQLEndpoint { batch, dl_id, .. } => {
                    let graphql_data_loader = GraphqlDataLoader::new(h_client.clone(), *batch)
                      .to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());

                    let _ = dl_id.insert(DataLoaderId(gql_data_loaders.len()));

                    gql_data_loaders.push(graphql_data_loader);
                  }

                  IO::Grpc { req_template, group_by, dl_id } => {
                    let data_loader = GrpcDataLoader {
                      client: h2_client.clone(),
                      operation: req_template.operation.clone(),
                      group_by: group_by.clone(),
                    };
                    let data_loader = data_loader.to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());

                    let _ = dl_id.insert(DataLoaderId(grpc_data_loaders.len()));

                    grpc_data_loaders.push(data_loader);
                  }
                  IO::JS(_, _) => {}
                };
              }
            })
          }
        }
      }
    }

    let schema = blueprint.to_schema();

    AppContext {
      schema,
      universal_http_client: h_client,
      http2_only_client: h2_client,
      blueprint,
      http_data_loaders: Arc::new(http_data_loaders),
      gql_data_loaders: Arc::new(gql_data_loaders),
      grpc_data_loaders: Arc::new(grpc_data_loaders),
      cache,
      env_vars: env,
    }
  }

  pub async fn execute(&self, request: impl Into<DynamicRequest>) -> Response {
    self.schema.execute(request).await
  }
}
