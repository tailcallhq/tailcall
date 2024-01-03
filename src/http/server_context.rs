use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dynamic;
use async_graphql_value::ConstValue;

use super::{DataLoaderRequest, DefaultHttpClient, HttpClient, HttpClientOptions};
use crate::blueprint::Type::ListType;
use crate::blueprint::{self, Blueprint, Definition};
use crate::chrono_cache::ChronoCache;
use crate::data_loader::DataLoader;
use crate::graphql::GraphqlDataLoader;
use crate::grpc;
use crate::grpc::data_loader::GrpcDataLoader;
use crate::http::HttpDataLoader;
use crate::lambda::{Cache, DataLoaderId, Expression, Unsafe};

pub struct ServerContext {
  pub schema: dynamic::Schema,
  pub universal_http_client: Arc<dyn HttpClient>,
  pub http2_only_client: Arc<dyn HttpClient>,
  pub blueprint: Blueprint,
  pub http_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, HttpDataLoader>>>,
  pub gql_data_loaders: Arc<Vec<DataLoader<DataLoaderRequest, GraphqlDataLoader>>>,
  pub cache: ChronoCache<u64, ConstValue>,
  pub grpc_data_loaders: Arc<Vec<DataLoader<grpc::DataLoaderRequest, GrpcDataLoader>>>,
  pub env_vars: Arc<HashMap<String, String>>,
}

impl ServerContext {
  pub fn new(blueprint: Blueprint) -> Self {
    let universal_http_client = Arc::new(DefaultHttpClient::new(&blueprint.upstream));
    let http2_only_client = Arc::new(DefaultHttpClient::with_options(
      &blueprint.upstream,
      HttpClientOptions { http2_only: true },
    ));

    Self::with_http_clients(blueprint, universal_http_client, http2_only_client)
  }

  pub fn with_http_clients(
    blueprint: Blueprint,
    universal_http_client: Arc<dyn HttpClient>,
    http2_only_client: Arc<dyn HttpClient>,
  ) -> Self {
    let http_data_loaders = vec![];
    let gql_data_loaders = vec![];
    let grpc_data_loaders = vec![];

    let schema = blueprint.to_schema();
    let env = std::env::vars().collect();
    let mut bp = blueprint.clone();

    let mut ctx = ServerContext {
      schema,
      universal_http_client,
      http2_only_client,
      blueprint,
      http_data_loaders: Arc::new(http_data_loaders),
      gql_data_loaders: Arc::new(gql_data_loaders),
      cache: ChronoCache::new(),
      grpc_data_loaders: Arc::new(grpc_data_loaders),
      env_vars: Arc::new(env),
    };

    for def in bp.definitions.iter_mut() {
      if let Definition::ObjectTypeDefinition(def) = def {
        for field in &mut def.fields {
          if let Some(expr) = &field.resolver {
            field.resolver = ctx.check_resolver_of_field(field, &expr);
          }
        }
      }
    }
    ctx.schema = bp.to_schema();
    ctx.blueprint = bp;
    ctx
  }

  fn check_resolver_of_field(&mut self, field: &blueprint::FieldDefinition, expr: &Expression) -> Option<Expression> {
    let bt = self.blueprint.upstream.batch.clone().unwrap_or_default();
    let universal_http_client = self.universal_http_client.clone();
    let http2_only_client = self.http2_only_client.clone();
    if let Expression::Unsafe(expr_unsafe) = &expr {
      match expr_unsafe {
        Unsafe::Http { req_template, group_by, .. } => {
          let data_loader = HttpDataLoader::new(
            universal_http_client.clone(),
            group_by.clone(),
            matches!(&field.of_type, ListType { .. }),
          )
          .to_data_loader(bt);

          let resolver = Some(Expression::Unsafe(Unsafe::Http {
            req_template: req_template.clone(),
            group_by: group_by.clone(),
            dl_id: Some(DataLoaderId(self.http_data_loaders.len())),
          }));

          let http_data_loaders = Arc::get_mut(&mut self.http_data_loaders).unwrap();
          http_data_loaders.push(data_loader);
          resolver
        }

        Unsafe::GraphQLEndpoint { req_template, field_name, batch, .. } => {
          let graphql_data_loader = GraphqlDataLoader::new(universal_http_client.clone(), *batch).to_data_loader(bt);

          let resolver = Some(Expression::Unsafe(Unsafe::GraphQLEndpoint {
            req_template: req_template.clone(),
            field_name: field_name.clone(),
            batch: *batch,
            dl_id: Some(DataLoaderId(self.gql_data_loaders.len())),
          }));

          let gql_data_loaders = Arc::get_mut(&mut self.gql_data_loaders).unwrap();
          gql_data_loaders.push(graphql_data_loader);
          resolver
        }

        Unsafe::Grpc { req_template, group_by, .. } => {
          let data_loader = GrpcDataLoader {
            client: http2_only_client.clone(),
            operation: req_template.operation.clone(),
            group_by: group_by.clone(),
          };
          let data_loader = data_loader.to_data_loader(bt);

          let resolver = Some(Expression::Unsafe(Unsafe::Grpc {
            req_template: req_template.clone(),
            group_by: group_by.clone(),
            dl_id: Some(DataLoaderId(self.grpc_data_loaders.len())),
          }));

          let grpc_data_loaders = Arc::get_mut(&mut self.grpc_data_loaders).unwrap();
          grpc_data_loaders.push(data_loader);
          resolver
        }
        _ => None,
      }
    } else if let Expression::Cache(cache) = &expr {
      let new_expr = self.check_resolver_of_field(field, cache.source());
      let resolver = if let Some(ne) = new_expr {
        Some(Expression::Cache(Cache::new(
          cache.hasher().clone(),
          cache.max_age(),
          Box::new(ne),
        )))
      } else {
        None
      };
      resolver
    } else {
      Some(expr.clone())
    }
  }
}
