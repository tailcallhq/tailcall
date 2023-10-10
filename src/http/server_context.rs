use std::sync::Arc;

use async_graphql::dataloader::{DataLoader, NoCache};
use async_graphql::dynamic;
use derive_setters::Setters;

use crate::blueprint::{Blueprint, Definition};
use crate::config::Server;
use crate::directive::DirectiveCodec;
use crate::group_by::GroupBy;
use crate::http::{DefaultHttpClient, HttpDataLoader};
use crate::lambda::{Expression, Operation};

#[derive(Setters, Clone)]
pub struct ServerContext {
  pub schema: dynamic::Schema,
  pub http_client: DefaultHttpClient,
  pub server: Server,
  pub data_loaders: Vec<Arc<DataLoader<HttpDataLoader<DefaultHttpClient>, NoCache>>>,
}

fn assign_id(blueprint: &mut Blueprint) -> &Blueprint {
  for (index, def) in blueprint.definitions.iter_mut().enumerate() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &mut def.fields {
        if let Some(Expression::Unsafe(Operation::Endpoint(req_template))) = &mut field.resolver {
          req_template.id = Some(index);
        }
      }
    }
  }
  blueprint
}

pub fn get_data_loaders(
  blueprint: &Blueprint,
  server: Server,
  http_client: DefaultHttpClient,
) -> Vec<Arc<DataLoader<HttpDataLoader<DefaultHttpClient>, NoCache>>> {
  let mut data_loaders = Vec::new();
  for def in blueprint.definitions.iter() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &def.fields {
        if let Some(Expression::Unsafe(Operation::Endpoint(_))) = &field.resolver {
          let mut data_loader = Arc::new(
            HttpDataLoader::new(http_client.clone(), None).to_data_loader(server.batch.clone().unwrap_or_default()),
          );
          field.directives.iter().for_each(|directive| {
            if directive.name == "batch" {
              let batched = GroupBy::from_directive(&directive.arguments.to_directive(directive.name.to_string()));
              data_loader = Arc::new(
                HttpDataLoader::new(http_client.clone(), batched.ok())
                  .to_data_loader(server.batch.clone().unwrap_or_default()),
              );
            }
          });

          data_loaders.push(data_loader);
        }
      }
    }
  }
  data_loaders
}

impl ServerContext {
  pub fn new(blueprint: &mut Blueprint, server: Server) -> Self {
    let schema = assign_id(blueprint).to_schema(&server);
    let http_client = DefaultHttpClient::new(server.clone());
    let data_loaders = get_data_loaders(blueprint, server.clone(), http_client.clone());
    ServerContext { schema, http_client, server: server.clone(), data_loaders }
  }
}
