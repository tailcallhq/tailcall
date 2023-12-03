use std::collections::BTreeMap;
use std::sync::Arc;

use async_graphql::dynamic;

use super::HttpClient;
use crate::blueprint::Type::ListType;
use crate::blueprint::{Blueprint, Definition};
use crate::http::{GraphqlDataLoader, HttpDataLoader};
use crate::lambda::{Expression, Unsafe, UrlToObjFieldsMap};

pub struct ServerContext {
  pub schema: dynamic::Schema,
  pub http_client: Arc<dyn HttpClient>,
  pub blueprint: Blueprint,
}

fn assign_data_loaders(blueprint: &mut Blueprint, http_client: Arc<dyn HttpClient>) -> &Blueprint {
  for def in blueprint.definitions.iter_mut() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &mut def.fields {
        if let Some(Expression::Unsafe(expr_unsafe)) = &mut field.resolver {
          match expr_unsafe {
            Unsafe::Http(req_template, group_by, _) => {
              let data_loader = HttpDataLoader::new(
                http_client.clone(),
                group_by.clone(),
                matches!(&field.of_type, ListType { .. }),
              )
              .to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());

              field.resolver = Some(Expression::Unsafe(Unsafe::Http(
                req_template.clone(),
                group_by.clone(),
                Some(Arc::new(data_loader)),
              )));
            }

            Unsafe::GraphQLEndpoint { req_template, field_name, batch, .. } => {
              let graphql_data_loader = GraphqlDataLoader::new(http_client.clone(), *batch)
                .to_data_loader(blueprint.upstream.batch.clone().unwrap_or_default());

              field.resolver = Some(Expression::Unsafe(Unsafe::GraphQLEndpoint {
                req_template: req_template.clone(),
                field_name: field_name.clone(),
                batch: *batch,
                data_loader: Some(Arc::new(graphql_data_loader)),
              }))
            }
            _ => {}
          }
        }
      }
    }
  }
  blueprint
}

fn assign_url_obj_fields(blueprint: &mut Blueprint) -> &mut Blueprint {
  let mut url_obj_fields: UrlToObjFieldsMap = BTreeMap::new();

  let all_upstream_graphql_urls = get_all_graphql_upstream_urls(blueprint);

  for def in blueprint.definitions.iter_mut() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &def.fields {
        let field_pair = (field.name.clone(), field.of_type.name().to_string());
        let urls = match &field.resolver {
          Some(Expression::Unsafe(Unsafe::GraphQLEndpoint { req_template, .. })) => vec![req_template.url.to_string()],
          _ => all_upstream_graphql_urls.clone(), // No resolver, add field pair for all urls
        };
        for url in urls {
          add_field_pair(&mut url_obj_fields, (&url, &def.name), field_pair.clone());
        }
      }
    }
  }

  for def in blueprint.definitions.iter_mut() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &mut def.fields {
        if let Some(Expression::Unsafe(Unsafe::GraphQLEndpoint { req_template, .. })) = &mut field.resolver {
          req_template.url_obj_fields = url_obj_fields.clone();
        }
      }
    }
  }
  blueprint
}

type PathToFields<'a> = (&'a String, &'a String);

fn add_field_pair(url_obj_fields: &mut UrlToObjFieldsMap, path: PathToFields, field_pair: (String, String)) {
  url_obj_fields
    .entry(path.0.clone())
    .and_modify(|obj_fields| {
      obj_fields
        .entry(path.1.clone())
        .and_modify(|fields| fields.push(field_pair.clone()))
        .or_insert(vec![field_pair.clone()]);
    })
    .or_insert(BTreeMap::from([(path.1.clone(), vec![field_pair.clone()])]));
}

fn get_all_graphql_upstream_urls(blueprint: &Blueprint) -> Vec<String> {
  let mut all_graphql_urls: Vec<String> = Vec::new();
  for def in blueprint.definitions.iter() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &def.fields {
        if let Some(Expression::Unsafe(Unsafe::GraphQLEndpoint { req_template, .. })) = &field.resolver {
          if !all_graphql_urls.contains(&req_template.url) {
            all_graphql_urls.push(req_template.url.clone());
          }
        }
      }
    }
  }
  all_graphql_urls
}

fn assign_url_obj_ids(blueprint: &mut Blueprint) -> &mut Blueprint {
  let url_obj_ids = get_url_obj_ids(blueprint);
  for def in blueprint.definitions.iter_mut() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &mut def.fields {
        if let Some(Expression::Unsafe(Unsafe::GraphQLEndpoint { req_template, .. })) = &mut field.resolver {
          req_template.url_obj_ids = url_obj_ids.clone();
        }
      }
    }
  }
  blueprint
}

fn get_url_obj_ids(blueprint: &Blueprint) -> BTreeMap<String, BTreeMap<String, Vec<String>>> {
  let mut url_obj_ids: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();

  for def in blueprint.definitions.iter() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &def.fields {
        if let Some(Expression::Unsafe(Unsafe::GraphQLEndpoint { req_template, .. })) = &field.resolver {
          if req_template.is_entities_query() {
            let id = req_template
              .operation_arguments
              .as_ref()
              .and_then(|args| args.first().map(|arg| arg.0.clone()));
            if let Some(id) = id {
              url_obj_ids
                .entry(req_template.url.clone())
                .and_modify(|obj_ids| {
                  obj_ids
                    .entry(req_template.parent_type_name.clone())
                    .and_modify(|ids| ids.push(id.clone()))
                    .or_insert(vec![id.clone()]);
                })
                .or_insert(BTreeMap::from([(
                  req_template.parent_type_name.clone(),
                  vec![id.clone()],
                )]));
            }
          }
        }
      }
    }
  }

  url_obj_ids
}

impl ServerContext {
  pub fn new(mut blueprint: Blueprint, http_client: Arc<dyn HttpClient>) -> Self {
    let blueprint = assign_url_obj_fields(&mut blueprint);
    let blueprint = assign_url_obj_ids(blueprint);
    let schema = assign_data_loaders(blueprint, http_client.clone()).to_schema();
    ServerContext { schema, http_client, blueprint: blueprint.clone() }
  }
}
