mod graphql_data_loader;

use std::collections::BTreeMap;

use async_graphql::futures_util::future::join_all;
pub use graphql_data_loader::*;
use reqwest;

const INTROSPECTION_QUERY: &str =
  "{\"query\":\"query { __schema { queryType { name fields { name args { name type { name kind ofType {name }}} type { name kind ofType { name fields {name} } fields { name }}  } } } }\"}";

// GraphQL introspection response types.
#[derive(serde::Deserialize, Debug, Clone)]
pub struct IntrospectionResult {
  pub data: Option<IntrospectionData>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct IntrospectionData {
  pub __schema: IntrospectionSchema,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct IntrospectionSchema {
  #[serde(rename = "queryType")]
  pub query_type: IntrospectionType,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct IntrospectionType {
  pub fields: Vec<Field>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Field {
  pub name: String,
  pub args: Option<Vec<Arg>>,
  #[serde(rename = "type")]
  pub type_: Option<Type_>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Arg {
  pub name: String,
  #[serde(rename = "type")]
  pub type_: Type_,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Type_ {
  pub name: Option<String>,
  pub kind: Option<String>,
  #[serde(rename = "ofType")]
  pub of_type: Option<Box<Type_>>,
  pub fields: Option<Vec<Field>>,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeKind {
  Scalar,
  Object,
  Interface,
  Union,
  Enum,
  InputObject,
  List,
  NonNull,
}

pub async fn introspect(graphql_url: &String) -> Result<IntrospectionResult, Box<dyn std::error::Error>> {
  let result = reqwest::Client::new()
    .post(graphql_url)
    .header("Content-Type", "application/json")
    .body(INTROSPECTION_QUERY)
    .send()
    .await?
    .json::<IntrospectionResult>()
    .await?;
  Ok(result)
}

pub async fn introspect_graphql_sources(graphql_urls: Vec<String>) -> BTreeMap<String, IntrospectionResult> {
  let mut results = BTreeMap::new();
  // let graphql_urls = graphql_urls.clone();
  let introspect_futures = graphql_urls
    .iter()
    .map(|url| async move { (url.clone(), introspect(url).await.unwrap()) });
  let joined = join_all(introspect_futures).await;
  for (url, introspection_result) in joined {
    results.insert(url.clone(), introspection_result.clone());
  }
  results
}

pub fn get_arg_type(
  introspection_result: &IntrospectionResult,
  query_name: &String,
  arg_name: &String,
) -> Option<String> {
  match introspection_result {
    IntrospectionResult {
      data: Some(IntrospectionData { __schema: IntrospectionSchema { query_type: IntrospectionType { fields } } }),
    } => {
      let introspect_query_field = fields.iter().find(|field| field.name == *query_name);
      match introspect_query_field {
        Some(field) => match &field.args {
          Some(args) => {
            let introspect_arg = args.iter().find(|arg| arg.name == *arg_name);
            match introspect_arg {
              Some(arg) => match &arg.type_.name {
                Some(name) => Some(name.clone()),
                None => {
                  let kind = arg.type_.kind.clone().unwrap_or_default();
                  match &arg.type_.of_type {
                    Some(type_) => match &type_.name {
                      Some(name) => {
                        if kind == "LIST" {
                          Some(format!("[{}]", name))
                        } else if kind == "NON_NULL" {
                          Some(format!("{}!", name))
                        } else {
                          Some(name.clone())
                        }
                      }
                      _ => None,
                    },
                    None => None,
                  }
                }
              },
              None => None,
            }
          }
          None => None,
        },
        None => None,
      }
    }
    _ => None,
  }
}
