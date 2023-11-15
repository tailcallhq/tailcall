use std::sync::Arc;

use async_graphql::dynamic;
use derive_setters::Setters;

use super::HttpClient;
use crate::blueprint::Type::ListType;
use crate::blueprint::{Blueprint, Definition};
use crate::http::{DataLoaderType, HttpListDataLoader, HttpObjectDataLoader};
use crate::lambda::{Expression, Unsafe};

#[derive(Setters, Clone)]
pub struct ServerContext {
  pub schema: dynamic::Schema,
  pub http_client: Arc<dyn HttpClient>,
  pub blueprint: Blueprint,
}

fn assign_data_loaders(blueprint: &mut Blueprint, http_client: Arc<dyn HttpClient>) -> &Blueprint {
  for def in blueprint.definitions.iter_mut() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &mut def.fields {
        if let Some(Expression::Unsafe(Unsafe::Http(req_template, group_by, _))) = &mut field.resolver {
          let data_loader_type = match &field.of_type {
            ListType { .. } => {
              let data_loader = HttpListDataLoader::new(http_client.clone(), group_by.clone())
                .to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());
              DataLoaderType::List(data_loader)
            }
            _ => {
              let data_loader = HttpObjectDataLoader::new(http_client.clone(), group_by.clone())
                .to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());
              DataLoaderType::Object(data_loader)
            }
          };
          field.resolver = Some(Expression::Unsafe(Unsafe::Http(
            req_template.clone(),
            group_by.clone(),
            Some(Arc::new(data_loader_type)),
          )));
        }
      }
    }
  }
  blueprint
}

impl ServerContext {
  pub fn new(blueprint: Blueprint, http_client: Arc<dyn HttpClient>) -> Self {
    let schema = assign_data_loaders(&mut blueprint.clone(), http_client.clone()).to_schema();
    ServerContext { schema, http_client, blueprint }
  }
}
