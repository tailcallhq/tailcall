#!/usr/bin/env rust-script

//! ```cargo
//! [dependencies]
//! tokio = {version = "1.35.1",features = ["macros"]}
//! env_logger = "0.10.1"
//! log = "0.4.20"
//! anyhow = "1.0.79"
//! schemars = "0.8.16"
//! serde_json = "1.0.111"
//! tailcall = {path = "."}
//! ```

use std::env;
use std::path::PathBuf;
use std::process::exit;

use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use tailcall::cli::init_file;
use tailcall::config::Config;
use tailcall::FileIO;

static JSON_SCHEMA_FILE: &'static str = ".tailcallrc.schema.json";

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
  let mut json_schema = PathBuf::from(file!());
  json_schema.pop();
  json_schema.push("examples");
  json_schema.push(JSON_SCHEMA_FILE);

  let file_io = init_file();
  let content = file_io
    .read(json_schema.to_str().ok_or(anyhow!("Unable to determine path"))?)
    .await?;
  let content = serde_json::from_str::<Value>(&content)?;
  let schema = get_updated_json().await?;
  match content.eq(&schema) {
    true => Ok(()),
    false => Err(anyhow!("Schema mismatch")),
  }
}

async fn mode_fix() -> Result<()> {
  update_json().await?;
  // update_gql().await?;
  Ok(())
}

async fn update_json() -> Result<()> {
  let mut json_schema = PathBuf::from(file!());
  json_schema.pop();
  json_schema.push("examples");
  json_schema.push(JSON_SCHEMA_FILE);

  let schema = serde_json::to_string_pretty(&get_updated_json().await?)?;
  let file_io = init_file();
  file_io
    .write(
      json_schema.to_str().ok_or(anyhow!("Unable to determine path"))?,
      schema.as_bytes(),
    )
    .await?;
  Ok(())
}

async fn get_updated_json() -> Result<Value> {
  let schema = schemars::schema_for!(Config);
  let schema = json!(schema);
  Ok(schema)
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
