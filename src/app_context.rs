use std::sync::Arc;

use async_graphql::dynamic::{self, DynamicRequest};
use async_graphql::Response;

use crate::blueprint::Type::ListType;
use crate::blueprint::{Blueprint, Definition};
use crate::data_loader::DataLoader;
use crate::graphql::GraphqlDataLoader;
use crate::grpc::data_loader::GrpcDataLoader;
use crate::http::{DataLoaderRequest, HttpDataLoader};
use crate::lambda::{Cache, DataLoaderId, Expression, IO};
use crate::{blueprint, grpc, EntityCache, EnvIO, HttpIO};

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
    blueprint: Blueprint,
    h_client: Arc<Http>,
    h2_client: Arc<Http>,
    env: Arc<Env>,
    cache: Arc<EntityCache>,
  ) -> Self {
    let mut bp = blueprint.clone();

    let mut ctx = AppContext {
      schema: blueprint.to_schema(),
      universal_http_client: h_client,
      http2_only_client: h2_client,
      blueprint,
      http_data_loaders: Arc::new(vec![]),
      gql_data_loaders: Arc::new(vec![]),
      cache,
      grpc_data_loaders: Arc::new(vec![]),
      env_vars: env,
    };

    for def in bp.definitions.iter_mut() {
      if let Definition::ObjectTypeDefinition(def) = def {
        for field in &mut def.fields {
          if let Some(expr) = field.resolver.clone() {
            ctx.check_resolver_of_field(field, &expr);
          }
        }
      }
    }

    ctx.schema = bp.to_schema();
    ctx.blueprint = bp;
    ctx
  }

  // return true means field is modified
  fn check_resolver_of_field(&mut self, field: &mut blueprint::FieldDefinition, expr: &Expression) -> bool {
    match expr {
      Expression::IO(expr_unsafe) => self.check_unsafe_expr(expr_unsafe, field),
      Expression::Cache(cache) => {
        let modified = self.check_resolver_of_field(field, cache.source());
        if modified {
          field.resolver = field.resolver.as_ref().map(|ne| {
            Expression::Cache(Cache::new(
              cache.hasher().clone(),
              cache.max_age(),
              Box::new(ne.clone()),
            ))
          });
        }
        modified
      }
      _ => false,
    }
  }

  // return true means field is modified
  fn check_unsafe_expr(&mut self, expr_unsafe: &IO, field: &mut blueprint::FieldDefinition) -> bool {
    let bt = self.blueprint.upstream.batch.clone().unwrap_or_default();
    let h_client = self.universal_http_client.clone();
    let h2_client = self.http2_only_client.clone();
    let mut modified = false;
    match expr_unsafe {
      IO::Http { req_template, group_by, .. } => {
        let data_loader = HttpDataLoader::new(
          h_client.clone(),
          group_by.clone(),
          matches!(&field.of_type, ListType { .. }),
        )
        .to_data_loader(bt);

        field.resolver = Some(Expression::IO(IO::Http {
          req_template: req_template.clone(),
          group_by: group_by.clone(),
          dl_id: Some(DataLoaderId(self.http_data_loaders.len())),
        }));
        modified = true;

        let http_data_loaders = Arc::get_mut(&mut self.http_data_loaders).unwrap();
        http_data_loaders.push(data_loader);
      }

      IO::GraphQLEndpoint { req_template, field_name, batch, .. } => {
        let graphql_data_loader = GraphqlDataLoader::new(h_client.clone(), *batch).to_data_loader(bt);

        field.resolver = Some(Expression::IO(IO::GraphQLEndpoint {
          req_template: req_template.clone(),
          field_name: field_name.clone(),
          batch: *batch,
          dl_id: Some(DataLoaderId(self.gql_data_loaders.len())),
        }));
        modified = true;

        let gql_data_loaders = Arc::get_mut(&mut self.gql_data_loaders).unwrap();
        gql_data_loaders.push(graphql_data_loader);
      }

      IO::Grpc { req_template, group_by, .. } => {
        let data_loader = GrpcDataLoader {
          client: h2_client.clone(),
          operation: req_template.operation.clone(),
          group_by: group_by.clone(),
        };
        let data_loader = data_loader.to_data_loader(bt);

        field.resolver = Some(Expression::IO(IO::Grpc {
          req_template: req_template.clone(),
          group_by: group_by.clone(),
          dl_id: Some(DataLoaderId(self.grpc_data_loaders.len())),
        }));
        modified = true;

        let grpc_data_loaders = Arc::get_mut(&mut self.grpc_data_loaders).unwrap();
        grpc_data_loaders.push(data_loader);
      }
      _ => {}
    }
    modified
  }

  pub async fn execute(&self, request: impl Into<DynamicRequest>) -> Response {
    self.schema.execute(request).await
  }
}
