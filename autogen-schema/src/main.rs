use std::env;
use std::path::PathBuf;
use std::process::exit;

use anyhow::{anyhow, Result};
use jsonschema::{Draft, JSONSchema};
use serde_json::{json, Value};
use tailcall::cli::init_file;
use tailcall::config::Config;
use tailcall::FileIO;

static JSON_SCHEMA_FILE: &'static str = ".tailcallrc.json";
static GQL_SCHEMA_FILE: &'static str = ".tailcallrc.graphql";

#[tokio::main]
async fn main() {
  logger_init();
  let args: Vec<String> = env::args().collect();
  let arg = args.get(1);

  if arg.is_none() {
    log::error!("An argument required, you can pass either `fix` or `check` argument");
    return;
  }
  match arg.unwrap().as_str() {
    "fix" => {
      let result = mode_fix().await;
      if let Err(e) = result {
        log::error!("{}", e);
        exit(1);
      }
      log::info!("JSON Schema updated in the file .tailcallrc.json");
    }
    "check" => {
      let result = mode_check().await;
      if let Err(e) = result {
        log::error!("{}", e);
        exit(1);
      }
      log::info!("The schema is valid.");
    }
    &_ => {
      log::error!("Unknown argument, you can pass either `fix` or `check` argument");
      return;
    }
  }
}

async fn mode_check() -> Result<()> {
  let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  root_dir.pop();
  root_dir.push("examples");
  let mut json_placeholder = root_dir.clone();
  json_placeholder.push("jsonplaceholder.json");

  let mut json_schema = root_dir.clone();
  json_schema.push(JSON_SCHEMA_FILE);

  let file_io = init_file();
  let content = file_io
    .read(json_schema.to_str().ok_or(anyhow!("Unable to determine path"))?)
    .await?;

  let compiled_schema = JSONSchema::options()
    .with_draft(Draft::Draft7)
    .compile(&serde_json::from_slice::<Value>(content.as_bytes())?)
    .map_err(|e| anyhow!(e.to_string()))?;

  let content = file_io
    .read(json_placeholder.to_str().ok_or(anyhow!("Unable to determine path"))?)
    .await?;

  let placeholder = serde_json::from_str::<Value>(&content).unwrap();
  compiled_schema.validate(&placeholder).map_err(|errs| {
    let errs = errs.map(|e| format!("{}\n", e)).collect::<Vec<String>>();
    anyhow!("{:?}", errs)
  })?;

  Ok(())
}

async fn mode_fix() -> Result<()> {
  update_json().await?;
  // update_gql().await?;
  Ok(())
}

async fn update_json() -> Result<()> {
  let mut json_schema = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  json_schema.pop();
  json_schema.push("examples");
  json_schema.push(JSON_SCHEMA_FILE);

  let schema = schemars::schema_for!(Config);
  let serde = json!(schema);
  let schema = serde_json::to_string_pretty(&serde)?;
  let file_io = init_file();
  file_io
    .write(
      json_schema.to_str().ok_or(anyhow!("Unable to determine path"))?,
      schema.as_bytes(),
    )
    .await?;
  Ok(())
}

async fn update_gql() -> Result<()> {
  unimplemented!()
}

fn logger_init() {
  // set the log level
  const LONG_ENV_FILTER_VAR_NAME: &str = "TAILCALL_SCHEMA_LOG_LEVEL";
  const SHORT_ENV_FILTER_VAR_NAME: &str = "TC_SCHEMA_LOG_LEVEL";

  // Select which env variable to use for the log level filter. This is because filter_or doesn't allow picking between multiple env_var for the filter value
  let filter_env_name = env::var(LONG_ENV_FILTER_VAR_NAME)
    .map(|_| LONG_ENV_FILTER_VAR_NAME)
    .unwrap_or_else(|_| SHORT_ENV_FILTER_VAR_NAME);

  // use the log level from the env if there is one, otherwise use the default.
  let env = env_logger::Env::new().filter_or(filter_env_name, "info");

  env_logger::Builder::from_env(env).init();
}
