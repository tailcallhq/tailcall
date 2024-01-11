use std::fs;
use std::path::PathBuf;
use std::sync::Once;

use derive_setters::Setters;
use regex::Regex;
use tailcall::blueprint::{Blueprint, validation_schema, validate_operation};
use tailcall::config::Config;
use tailcall::valid::Valid;

static INIT: Once = Once::new();

const SERVER_SDL: &str = "server-sdl";
const CLIENT_QUERY: &str = "client-query";
const SPEC_ONLY: &str = "spec-only";
const SPEC_SKIP: &str = "spec-skip";
const SPEC_FAIL: &str = "spec-fail";

#[derive(Debug)]
enum Annotation {
  Skip,
  Only,
  Fail,
}

#[derive(Debug, Default, Setters)]
struct OperationSpec {
  path: PathBuf,
  server_sdl: String,
  test_queries: Vec<OperationQuerySpec>,
  annotation: Option<Annotation>,
}

#[derive(Debug)]
struct OperationQuerySpec {
  query: String,
  diagnostic_count: u32,
}

impl OperationSpec {
  fn query(mut self, query: String, diagnostic_count: u32) -> Self {
    self.test_queries.push(OperationQuerySpec { query, diagnostic_count });
    self
  }

  fn new(path: PathBuf, content: &str) -> OperationSpec {
    INIT.call_once(|| {
      env_logger::builder()
        .filter(Some("operation_spec"), log::LevelFilter::Info)
        .init();
    });

    let mut spec = OperationSpec::default().path(path);
    for component in content.split("#>") {
      if component.contains(SPEC_ONLY) {
        spec = spec.annotation(Some(Annotation::Only));
      }
      if component.contains(SPEC_SKIP) {
        spec = spec.annotation(Some(Annotation::Skip));
      }
      if component.contains(SPEC_FAIL) {
        spec = spec.annotation(Some(Annotation::Fail));
      }
      if component.contains(SERVER_SDL) {
        spec = spec.server_sdl(component.replace(SERVER_SDL, "").trim().to_string());
      }
      if component.contains(CLIENT_QUERY) {
        let regex = Regex::new(r"@diagnostic.*\) ").unwrap();
        let query_string = component.replace(CLIENT_QUERY, "");
        let parsed_query = async_graphql::parser::parse_query(query_string.clone()).unwrap();

        let query_string = regex.replace_all(query_string.as_str(), "");
        let query_string = query_string.trim();

        for (_, q) in parsed_query.operations.iter() {
          let diagnostic = q.node.directives.iter().find(|d| d.node.name.node == "diagnostic");
          assert!(
            diagnostic.is_some(),
            "@diagnostic directive is required in query:\n```\n{}\n```",
            query_string
          );
          if let Some(dir) = diagnostic {
            let diagnostic_count = dir
              .node
              .arguments
              .iter()
              .find(|a| a.0.node == "count")
              .map(|a| a.clone().1.node.to_string().parse::<u32>().unwrap())
              .unwrap();
            spec = spec.query(query_string.to_string(), diagnostic_count);
          }
        }
      }
    }
    spec
  }

  fn cargo_read(path: &str) -> std::io::Result<Vec<OperationSpec>> {
    let mut directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    directory.push(path);

    let entries = fs::read_dir(directory.clone())?;
    let mut files = Vec::new();
    let mut only_files = Vec::new();

    for entry in entries {
      let path = entry?.path();
      if path.is_file() && path.extension().unwrap_or_default() == "graphql" {
        let contents = fs::read_to_string(path.clone())?;
        let path_buf = path.clone();
        let spec = OperationSpec::new(path_buf, contents.as_str());

        match spec.annotation {
          Some(Annotation::Skip) => log::warn!("{} ... skipped", spec.path.display()),
          Some(Annotation::Only) => only_files.push(spec),
          Some(Annotation::Fail) | None => files.push(spec),
        }
      }
    }

    assert!(
      !files.is_empty() || !only_files.is_empty(),
      "No files found in {}",
      directory.to_str().unwrap_or_default()
    );

    if !only_files.is_empty() {
      Ok(only_files)
    } else {
      Ok(files)
    }
  }
}

#[tokio::test]
async fn test_operations() -> std::io::Result<()> {
  let specs = OperationSpec::cargo_read("tests/graphql/operations");

  let tasks: Vec<_> = specs?
    .into_iter()
    .map(|spec| {
      tokio::spawn(async move {
        let mut config = Config::from_sdl(spec.server_sdl.as_str()).to_result().unwrap();
        config.server.query_validation = Some(false);

        let blueprint = Valid::from(Blueprint::try_from(&config))
          .trace(spec.path.to_str().unwrap_or_default())
          .to_result()
          .unwrap();

        let schema = Valid::from(validation_schema(&blueprint))
          .trace(spec.path.to_str().unwrap_or_default())
          .to_result()
          .unwrap();

        for query_spec in spec.test_queries {
          let count = validate_operation(&schema, query_spec.query.as_str()).await.len() as u32;

          if spec.annotation.as_ref().is_some_and(|a| matches!(a, Annotation::Fail)) {
            assert_ne!(
              count,
              query_spec.diagnostic_count,
              "QueryExecution: {} (diagnostics should mismatch (got {})",
              spec.path.display(),
              query_spec.diagnostic_count
            );
          } else {
            assert_eq!(
              count,
              query_spec.diagnostic_count,
              "QueryExecution: {} (diagnostics count mismatch ({} expected but got {}))",
              spec.path.display(),
              query_spec.diagnostic_count,
              count
            );
          }
          log::info!("QueryExecution: {} ... ok", spec.path.display());
        }
      })
    })
    .collect();

  for task in tasks {
    task.await?;
  }
  Ok(())
}
