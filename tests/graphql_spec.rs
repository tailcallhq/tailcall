use std::fmt::Debug;
#[cfg(test)]
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use async_graphql::parser::types::TypeSystemDefinition;
use async_graphql::Request;
use derive_setters::Setters;
use hyper::http::{HeaderName, HeaderValue};
use hyper::HeaderMap;
use pretty_assertions::assert_eq;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tailcall::blueprint::Blueprint;
use tailcall::config::Config;
use tailcall::directive::DirectiveCodec;
use tailcall::http::{HttpDataLoader, RequestContext};
use tailcall::print_schema;
use tailcall::valid::Cause;

mod graphql_mock;

#[derive(Debug, Default, Setters)]
struct GraphQLSpec {
  path: PathBuf,
  client_sdl: String,
  server_sdl: String,
  sdl_errors: Vec<SDLError>,
  test_queries: Vec<GraphQLQuerySpec>,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
struct SDLError {
  message: String,
  trace: Vec<String>,
}

impl<'a> From<Cause<&'a str>> for SDLError {
  fn from(value: Cause<&'a str>) -> Self {
    SDLError { message: value.message.to_string(), trace: value.trace.iter().map(|e| e.to_string()).collect() }
  }
}

impl From<Cause<String>> for SDLError {
  fn from(value: Cause<String>) -> Self {
    SDLError { message: value.message.to_string(), trace: value.trace.iter().map(|e| e.to_string()).collect() }
  }
}

#[derive(Debug, Default)]
struct GraphQLQuerySpec {
  query: String,
  expected: Value,
}

impl GraphQLSpec {
  fn query(mut self, query: String, expected: Value) -> Self {
    self.test_queries.push(GraphQLQuerySpec { query, expected });
    self
  }

  fn new(path: PathBuf, content: &str) -> GraphQLSpec {
    let mut spec = GraphQLSpec::default().path(path);
    for component in content.split("#>") {
      if component.contains(CLIENT_SDL) {
        let trimmed = component.replace(CLIENT_SDL, "").trim().to_string();

        // Extract all errors
        if trimmed.contains("@error") {
          let doc = async_graphql::parser::parse_schema(trimmed.as_str()).unwrap();
          for def in doc.definitions {
            if let TypeSystemDefinition::Type(type_def) = def {
              for dir in type_def.node.directives {
                if dir.node.name.node == "error" {
                  spec.sdl_errors.push(SDLError::from_directive(&dir.node).unwrap());
                }
              }
            }
          }
        }

        spec = spec.client_sdl(trimmed);
      }
      if component.contains(SERVER_SDL) {
        spec = spec.server_sdl(component.replace(SERVER_SDL, "").trim().to_string());
      }
      if component.contains(CLIENT_QUERY) {
        let regex = Regex::new(r"@expect.*\) ").unwrap();
        let query_string = component.replace(CLIENT_QUERY, "");
        let parsed_query = async_graphql::parser::parse_query(query_string.clone()).unwrap();

        let query_string = regex.replace_all(query_string.as_str(), "");
        let query_string = query_string.trim();
        for (_, q) in parsed_query.operations.iter() {
          let expect = q.node.directives.iter().find(|d| d.node.name.node == "expect");
          assert!(
            expect.is_some(),
            "@expect directive is required in query:\n```\n{}\n```",
            query_string
          );
          if let Some(dir) = expect {
            let expected = dir
              .node
              .arguments
              .iter()
              .find(|a| a.0.node == "json")
              .map(|a| a.clone().1.node.into_json().unwrap())
              .unwrap();
            spec = spec.query(query_string.to_string(), expected);
          }
        }
      }
    }
    spec
  }

  fn cargo_read(path: &str) -> std::io::Result<Vec<GraphQLSpec>> {
    let mut dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir_path.push(path);

    let entries = fs::read_dir(dir_path.clone())?;
    let mut files = Vec::new();
    for entry in entries {
      let path = entry?.path();
      if path.is_file() && path.extension().unwrap_or_default() == "graphql" {
        let contents = fs::read_to_string(path.clone())?;
        let path_buf = path.clone();
        files.push(GraphQLSpec::new(path_buf, contents.as_str()));
      }
    }

    assert!(
      !files.is_empty(),
      "No files found in {}",
      dir_path.to_str().unwrap_or_default()
    );
    Ok(files)
  }
}

const CLIENT_SDL: &str = "client-sdl";
const SERVER_SDL: &str = "server-sdl";
const CLIENT_QUERY: &str = "client-query";

// Check if SDL -> Config -> SDL is identity
#[test]
fn test_config_identity() -> std::io::Result<()> {
  let specs = GraphQLSpec::cargo_read("tests/graphql");

  for spec in specs? {
    let content = spec.server_sdl;
    let expected = content.clone();

    let config = Config::from_sdl(content.as_str()).unwrap();
    let actual = config.to_sdl();
    assert_eq!(actual, expected, "SDL-Config identity failure: {}", spec.path.display());
  }

  Ok(())
}

// Check server SDL matches expected client SDL
#[test]
fn test_server_to_client_sdl() -> std::io::Result<()> {
  let specs = GraphQLSpec::cargo_read("tests/graphql");

  for spec in specs? {
    let expected = spec.client_sdl;
    let content = spec.server_sdl;
    let config = Config::from_sdl(content.as_str()).unwrap();
    let actual = print_schema::print_schema((Blueprint::try_from(&config).unwrap()).to_schema(&config.server));
    assert_eq!(
      actual,
      expected,
      "Server to client SDL failure: {}",
      spec.path.display()
    );
  }

  Ok(())
}

// Check if execution gives expected response
#[tokio::test]
async fn test_execution() -> std::io::Result<()> {
  let mut mock_server = graphql_mock::start_mock_server();
  graphql_mock::setup_mocks(&mut mock_server);

  let specs = GraphQLSpec::cargo_read("tests/graphql/passed");

  for spec in specs? {
    let mut config = Config::from_sdl(&spec.server_sdl).unwrap();
    config.server.enable_query_validation = Some(false);

    let blueprint = Blueprint::try_from(&config).unwrap();
    let schema = blueprint.to_schema(&config.server);

    for q in spec.test_queries {
      let mut headers = HeaderMap::new();
      headers.insert(HeaderName::from_static("authorization"), HeaderValue::from_static("1"));

      let data_loader = HttpDataLoader::default().to_async_data_loader();
      let req_ctx = RequestContext::default()
        .req_headers(headers)
        .server(config.server.clone())
        .data_loader(data_loader);
      let req = Request::from(q.query.as_str()).data(Arc::new(req_ctx));
      let res = schema.execute(req).await;
      let json = serde_json::to_string(&res).unwrap();
      let expected = serde_json::to_string(&q.expected).unwrap();
      assert_eq!(json, expected, "execution failure: {}", spec.path.display());
    }
  }

  Ok(())
}

// Standardize errors on Client SDL
#[test]
fn test_failures_in_client_sdl() -> std::io::Result<()> {
  let specs = GraphQLSpec::cargo_read("tests/graphql/errors");

  for spec in specs? {
    let expected = spec.sdl_errors;
    let content = spec.server_sdl;
    let config = Config::from_sdl(content.as_str());

    let config = match config {
      Ok(config) => config,
      Err(cause) => {
        assert_eq!(
          cause.to_string(),
          expected.first().unwrap().message,
          "Server SDL failure mismatch: {}",
          spec.path.display()
        );
        continue;
      }
    };

    let actual = Blueprint::try_from(&config);
    match actual {
      Err(cause) => {
        let actual: Vec<SDLError> = cause.as_vec().iter().map(|e| e.to_owned().into()).collect();
        assert_eq!(actual, expected, "Server SDL failure mismatch: {}", spec.path.display());
      }
      _ => panic!("Expected error not found: {}", spec.path.display()),
    }
  }

  Ok(())
}
