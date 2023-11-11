use std::sync::Arc;

use anyhow::anyhow;
use async_graphql::dynamic;
use derive_setters::Setters;

use super::HttpClient;
use crate::blueprint::{Blueprint, Definition};
use crate::config::config_poll::ConfigLoader;
use crate::http::HttpDataLoader;
use crate::lambda::{Expression, Unsafe};

#[derive(Setters)]
pub struct ServerContext {
  pub schema: SchemaLoader,
  pub http_client: Arc<dyn HttpClient>,
  pub blueprint: Blueprint,
}

pub enum SchemaLoader {
  Static(dynamic::Schema),
  Dynamic(ConfigLoader),
}
impl SchemaLoader {
  pub fn new_schema(schema: dynamic::Schema) -> Self {
    Self::Static(schema)
  }
  pub fn new_config(config: ConfigLoader) -> Self {
    Self::Dynamic(config)
  }

  pub fn get_schema(&self) -> anyhow::Result<&dynamic::Schema> {
    if let SchemaLoader::Static(schema) = self {
      Ok(schema)
    } else {
      Err(anyhow!("Not a static schema"))
    }
  }

  pub fn get_conf(&self) -> anyhow::Result<&ConfigLoader> {
    if let SchemaLoader::Dynamic(conf) = self {
      Ok(conf)
    } else {
      Err(anyhow!("Not a static schema"))
    }
  }
}

fn assign_data_loaders(blueprint: &mut Blueprint, http_client: Arc<dyn HttpClient>) -> &Blueprint {
  for def in blueprint.definitions.iter_mut() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &mut def.fields {
        if let Some(Expression::Unsafe(Unsafe::Http(req_template, group_by, _))) = &mut field.resolver {
          let data_loader = HttpDataLoader::new(http_client.clone(), group_by.clone())
            .to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());
          field.resolver = Some(Expression::Unsafe(Unsafe::Http(
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
  pub fn new(blueprint: Blueprint, http_client: Arc<dyn HttpClient>) -> Self {
    let schema = SchemaLoader::new_schema(assign_data_loaders(&mut blueprint.clone(), http_client.clone()).to_schema());
    ServerContext { schema, http_client, blueprint }
  }
}
