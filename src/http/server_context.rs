use std::sync::Arc;

use async_graphql::dataloader::{DataLoader, NoCache};
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
  pub data_loader: Arc<DataLoader<HttpDataLoader<DefaultHttpClient>, NoCache>>,
  pub data_loaders: Vec<Arc<DataLoader<HttpDataLoader<DefaultHttpClient>, NoCache>>>,
}

fn assign_id(blueprint: &mut Blueprint) -> &Blueprint {
  for (index, def) in blueprint.definitions.iter_mut().enumerate() {
    match def {
      Definition::ObjectTypeDefinition(def) => {
        for field in def.fields.iter_mut() {
          if let Some(ref mut resolver) = field.resolver {
            match resolver {
              Expression::Unsafe(Operation::Endpoint(ref mut req_template)) => {
                req_template.id = Some(index);
              }
              _ => {}
            }
          }
        }
      }
      _ => {}
    }
  }
  blueprint
}

impl ServerContext {
  pub fn new(blueprint: Blueprint, server: Server) -> Self {
    let mut blueprint = blueprint;
    let schema = assign_id(&mut blueprint).to_schema(&server);
    let http_client = DefaultHttpClient::new(server.clone());
    let mut data_loaders = Vec::new();
    for _ in blueprint.endpoints() {
      let data_loader =
        Arc::new(HttpDataLoader::new(http_client.clone()).to_data_loader(server.batch.clone().unwrap_or_default()));
      data_loaders.push(data_loader);
    }
    let data_loader = HttpDataLoader::new(http_client.clone()).to_data_loader(server.batch.clone().unwrap_or_default());
    ServerContext { schema, http_client, server: server.clone(), data_loader: Arc::new(data_loader), data_loaders }
  }
}
