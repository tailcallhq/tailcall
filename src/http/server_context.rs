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
              }));
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

  for url in all_upstream_graphql_urls.clone() {
    url_obj_fields.insert(url.clone(), BTreeMap::new());
  }
  for def in blueprint.definitions.iter_mut() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &def.fields {
        let field_pair = (field.name.clone(), field.of_type.name().to_string());
        if let Some(Expression::Unsafe(Unsafe::GraphQLEndpoint { req_template, .. })) = &field.resolver {
          update_fields(&mut url_obj_fields, (&req_template.url, &def.name), field_pair);
        } else {
          // no resolver, add field pair for all urls
          let all_upstream_graphql_urls = all_upstream_graphql_urls.clone();
          for url in all_upstream_graphql_urls {
            let field_pair = (field.name.clone(), field.of_type.name().to_string());
            update_fields(&mut url_obj_fields, (&url, &def.name), field_pair);
          }
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

fn update_fields(url_obj_fields: &mut UrlToObjFieldsMap, path: PathToFields, field_pair: (String, String)) {
  let obj_fields = url_obj_fields.get_mut(path.0);
  if let Some(obj_fields) = obj_fields {
    if let Some(fields) = obj_fields.get_mut(path.1) {
      fields.push(field_pair);
    } else {
      let fields = vec![field_pair];
      obj_fields.insert(path.1.clone(), fields);
    }
  }
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
          if req_template.federate && url_obj_ids.get(&req_template.url).is_none() {
            url_obj_ids.insert(req_template.url.clone(), BTreeMap::new());
          }
        }
      }
    }
  }

  for def in blueprint.definitions.iter() {
    if let Definition::ObjectTypeDefinition(def) = def {
      for field in &def.fields {
        if let Some(Expression::Unsafe(Unsafe::GraphQLEndpoint { req_template, .. })) = &field.resolver {
          if req_template.federate {
            let id = match &req_template.operation_arguments {
              Some(args) => {
                if !args.is_empty() {
                  args.first().map(|arg| arg.0.clone())
                } else {
                  None
                }
              }
              None => None,
            };
            if let Some(obj_ids) = url_obj_ids.get_mut(&req_template.url) {
              if let Some(ids) = obj_ids.get_mut(&req_template.parent_type_name) {
                if let Some(id) = id {
                  ids.push(id)
                }
              } else {
                let mut ids: Vec<String> = Vec::new();
                if let Some(id) = id {
                  ids.push(id)
                }
                obj_ids.insert(req_template.parent_type_name.clone(), ids);
              }
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
