use std::collections::BTreeMap;

use async_graphql::futures_util::future::join_all;
use reqwest;
use serde_json::json;

// GraphQL introspection response types.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct IntrospectionResult {
  pub data: Option<IntrospectionData>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct IntrospectionData {
  pub __schema: IntrospectionSchema,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct IntrospectionSchema {
  #[serde(rename = "queryType")]
  pub query_type: IntrospectionType,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct IntrospectionType {
  pub fields: Vec<Field>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Field {
  pub name: String,
  pub args: Option<Vec<Arg>>,
  #[serde(rename = "type")]
  pub type_: Option<Type_>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Arg {
  pub name: String,
  #[serde(rename = "type")]
  pub type_: Type_,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Type_ {
  pub name: Option<String>,
  pub kind: Option<String>,
  #[serde(rename = "ofType")]
  pub of_type: Option<Box<Type_>>,
  pub fields: Option<Vec<Field>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, PartialEq)]
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub struct IntrospectionResults(pub BTreeMap<String, IntrospectionResult>);

impl IntrospectionResults {
  pub fn merge_right(mut self, other: Self) -> Self {
    let mut merged = self.0;
    for (key, value) in other.0 {
      merged.insert(key, value);
    }
    self.0 = merged;
    self
  }
}

pub async fn introspect_endpoint(graphql_url: &String) -> Result<IntrospectionResult, Box<dyn std::error::Error>> {
  let introspection_query: String =
  json!({"query": "query { __schema { queryType { name fields { name args { name type { name kind ofType {name }}} type { name kind ofType { name fields {name} } fields { name }}  } } } }"}).to_string();

  let result = reqwest::Client::new()
    .post(graphql_url)
    .header("Content-Type", "application/json")
    .body(introspection_query)
    .send()
    .await?
    .json::<IntrospectionResult>()
    .await?;

  Ok(result)
}

pub async fn introspect_endpoints(graphql_urls: Vec<String>) -> IntrospectionResults {
  let mut results = BTreeMap::new();
  let introspect_futures = graphql_urls
    .iter()
    .map(|url| async move { (url.clone(), introspect_endpoint(url).await.unwrap()) });
  let joined = join_all(introspect_futures).await;
  for (url, introspection_result) in joined {
    results.insert(url.clone(), introspection_result.clone());
  }
  IntrospectionResults(results)
}

pub fn get_arg_type(
  introspection_result: &IntrospectionResult,
  query_name: &String,
  arg_name: &String,
) -> Option<String> {
  introspection_result
    .data
    .as_ref()
    .and_then(|data| {
      data
        .__schema
        .query_type
        .fields
        .iter()
        .find(|field| field.name == *query_name)
    })
    .and_then(|field| field.args.as_ref())
    .and_then(|args| args.iter().find(|arg| arg.name == *arg_name))
    .and_then(|arg| {
      arg.type_.name.as_ref().map_or_else(
        || {
          let kind = arg.type_.kind.clone().unwrap_or_default();
          match &arg.type_.of_type {
            Some(type_) => type_.name.as_ref().map(|name| format_name(name, &kind)),
            None => None,
          }
        },
        |name| Some(name.clone()),
      )
    })
}

fn format_name(name: &String, kind: &String) -> String {
  if kind == "LIST" {
    format!("[{}]", name)
  } else if kind == "NON_NULL" {
    format!("{}!", name)
  } else {
    name.clone()
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use httpmock::Method::POST;
  use httpmock::MockServer;

  use crate::config::introspection::*;

  fn load_mock_introspection_result() -> String {
    let mut mock_result_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    mock_result_path.push("tests/data/introspection-result.json");
    fs::read_to_string(mock_result_path).unwrap()
  }

  fn mock_introspection_results() -> BTreeMap<String, IntrospectionResult> {
    let result: IntrospectionResult = serde_json::from_str(&load_mock_introspection_result()).unwrap();
    let mut introspection_result: BTreeMap<String, IntrospectionResult> = BTreeMap::new();
    introspection_result.insert("http://localhost:8000/graphql".to_string(), result);
    introspection_result
  }

  #[test]
  fn test_get_arg_type() {
    let introspection_results = mock_introspection_results();
    let introspection_result = introspection_results
      .get(&"http://localhost:8000/graphql".to_string())
      .unwrap();
    assert_eq!(
      get_arg_type(introspection_result, &"user".to_string(), &"id".to_string()).unwrap(),
      "Int"
    );
    assert_eq!(
      get_arg_type(introspection_result, &"post".to_string(), &"id".to_string()).unwrap(),
      "Int!"
    );
    assert_eq!(
      get_arg_type(introspection_result, &"post".to_string(), &"doesntexist".to_string()),
      None
    );
    assert_eq!(
      get_arg_type(introspection_result, &"doesntexist".to_string(), &"id".to_string()),
      None
    );
  }

  #[tokio::test]
  async fn test_introspect_endpoint() {
    let server = MockServer::start();
    let result = load_mock_introspection_result();

    server.mock(|when, then| {
      when.method(POST).path("/graphql");
      then.status(200).header("content-type", "application/json").body(result);
    });

    let graphql_url = server.url("/graphql");

    let introspect_result = introspect_endpoint(&graphql_url).await;
    assert_eq!(
      introspect_result
        .unwrap()
        .data
        .unwrap()
        .__schema
        .query_type
        .fields
        .len(),
      2
    );
  }
}
