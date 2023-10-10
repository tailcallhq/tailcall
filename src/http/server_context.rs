use std::sync::Arc;

use async_graphql::dynamic;
use derive_setters::Setters;

use crate::blueprint::{Blueprint, Definition};
use crate::config::Server;
use crate::http::{DefaultHttpClient, HttpDataLoader};
use crate::lambda::{Expression, Operation};

#[derive(Setters, Clone)]
pub struct ServerContext {
  pub schema: dynamic::Schema,
  pub http_client: DefaultHttpClient,
  pub server: Server,
}

fn assign_data_loaders(blueprint: &mut Blueprint, server: Server, http_client: DefaultHttpClient) -> &Blueprint {
  for def in blueprint.definitions.iter_mut() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &mut def.fields {
        if let Some(Expression::Unsafe(Operation::Endpoint(req_template, group_by, _))) = &mut field.resolver {
          let data_loader = HttpDataLoader::new(http_client.clone(), group_by.clone())
            .to_data_loader(server.batch.clone().unwrap_or_default());
          field.resolver = Some(Expression::Unsafe(Operation::Endpoint(
            req_template.clone(),
            group_by.clone(),
            Some(Arc::new(data_loader)),
          )));
        }
      }
    }
  }
  blueprint
}

impl ServerContext {
  pub fn new(blueprint: &mut Blueprint, server: Server) -> Self {
    let http_client = DefaultHttpClient::new(server.clone());
    let schema = assign_data_loaders(blueprint, server.clone(), http_client.clone()).to_schema(&server);
    ServerContext { schema, http_client, server: server.clone() }
  }
}
