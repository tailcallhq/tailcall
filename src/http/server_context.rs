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

pub fn endpoints(blueprint: &Blueprint) -> Vec<&crate::request_template::RequestTemplate> {
  blueprint
    .definitions
    .iter()
    .filter_map(|def| match def {
      Definition::ObjectTypeDefinition(def) => Some(&def.fields),
      _ => None,
    })
    .flat_map(|fields| fields.iter())
    .filter_map(|field| match &field.resolver {
      Some(Expression::Unsafe(Operation::Endpoint(req_template))) => Some(req_template),
      _ => None,
    })
    .collect()
}

pub fn data_loaders(
  blueprint: &Blueprint,
  server: Server,
) -> Vec<Arc<DataLoader<HttpDataLoader<DefaultHttpClient>, NoCache>>> {
  println!("data loaders");
  let mut data_loaders = Vec::new();
  for def in blueprint.definitions.iter() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &def.fields {
        if let Some(Expression::Unsafe(Operation::Endpoint(req_template))) = &field.resolver {
          let mut data_loader = Arc::new(
            HttpDataLoader::new(DefaultHttpClient::default(), false)
              .to_data_loader(server.batch.clone().unwrap_or_default()),
          );
          println!("directive: {:?}", field.directives);
          field.directives.iter().for_each(|directive| {
            if directive.name == "batch" {
              println!("batch");
              data_loader = Arc::new(
                HttpDataLoader::new(DefaultHttpClient::default(), true)
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
  pub fn new(blueprint: Blueprint, server: Server) -> Self {
    let mut blueprint = blueprint;
    let schema = assign_id(&mut blueprint).to_schema(&server);
    let http_client = DefaultHttpClient::new(server.clone());
    let data_loaders = data_loaders(&blueprint, server.clone());
    ServerContext { schema, http_client, server: server.clone(), data_loaders }
  }
}
