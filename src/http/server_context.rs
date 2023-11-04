use std::sync::Arc;

use async_graphql::dynamic;
use derive_setters::Setters;

use crate::blueprint::{Blueprint, Definition};
use crate::http::{DefaultHttpClient, GraphqlDataLoader, HttpDataLoader};
use crate::lambda::{Expression, Operation};

#[derive(Setters, Clone)]
pub struct ServerContext {
  pub schema: dynamic::Schema,
  pub http_client: DefaultHttpClient,
  pub blueprint: Blueprint,
}

fn assign_data_loaders(blueprint: &mut Blueprint, http_client: DefaultHttpClient) -> &Blueprint {
  for def in blueprint.definitions.iter_mut() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &mut def.fields {
        if let Some(Expression::Unsafe(Operation::Endpoint(req_template, group_by, _))) = &mut field.resolver {
          let data_loader = HttpDataLoader::new(http_client.clone(), group_by.clone())
            .to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());
          field.resolver = Some(Expression::Unsafe(Operation::Endpoint(
            req_template.clone(),
            group_by.clone(),
            Some(Arc::new(data_loader)),
          )));
        }
        if let Some(Expression::Unsafe(Operation::GraphQLEndpoint(req_template, field_name, _))) = &mut field.resolver {
          let graphql_data_loader = GraphqlDataLoader::new(http_client.clone())
            .to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());
          field.resolver = Some(Expression::Unsafe(Operation::GraphQLEndpoint(
            req_template.clone(),
            field_name.clone(),
            Some(Arc::new(graphql_data_loader)),
          )))
        }
      }
    }
  }
  blueprint
}

impl ServerContext {
  pub fn new(blueprint: Blueprint) -> Self {
    let http_client = DefaultHttpClient::new(blueprint.upstream.clone());
    let schema = assign_data_loaders(&mut blueprint.clone(), http_client.clone()).to_schema();
    ServerContext { schema, http_client, blueprint }
  }
}
